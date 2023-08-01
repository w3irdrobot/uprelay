use crate::{db::DbPool, models::Relay};
use ::time::OffsetDateTime;
use anyhow::Result;
use cached::{Cached, TimedSizedCache};
use futures::{future, stream::StreamExt};
use log::{debug, error, info};
use nostr_sdk::nostr::nips::nip65::get_relay_list;
use nostr_sdk::prelude::*;
use reqwest::{
    header::{HeaderMap, ACCEPT},
    Client as HttpClient,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_stream::wrappers::BroadcastStream;
use url::Url;

const SECS_IN_NINETY_DAYS: u64 = 14 * 24 * 60 * 60;

pub struct Nostr {
    relay_repo: RelayContainer,
    nostr_client: Client,
}

impl Nostr {
    pub async fn new(relay_repo: RelayContainer) -> Result<Self> {
        let keys = Keys::generate();
        let nostr_client = Client::new(&keys);

        for url in vec![
            "wss://no.str.cr",
            "wss://nostr.bitcoiner.social",
            "wss://relay.snort.social",
            "wss://relay.damus.io",
        ] {
            info!("connecting to relay {}", url);
            nostr_client
                .add_relay(url, None)
                .await
                .expect(&format!("{} connects", url));
        }
        nostr_client.connect().await;
        info!("seed relays connected");

        Ok(Nostr {
            relay_repo,
            nostr_client,
        })
    }

    pub async fn hydrate(&self) -> Result<()> {
        let filter = Filter::new()
            .kinds(vec![Kind::RelayList])
            // get all starting 90 days ago
            .since(Timestamp::now() - Duration::from_secs(SECS_IN_NINETY_DAYS));
        self.nostr_client.subscribe(vec![filter]).await;

        Ok(BroadcastStream::new(self.nostr_client.notifications())
            .filter_map(|x| future::ready(x.ok()))
            .filter_map(|x| async {
                match x {
                    RelayPoolNotification::Event(_, e) => Some(e),
                    _ => None,
                }
            })
            .for_each_concurrent(15, |e| async move {
                debug!("handling event {:?}", e);
                let urls = match e.kind {
                    Kind::RelayList => get_relay_list(e)
                        .iter()
                        .filter_map(|r| Url::parse(&r.0).ok())
                        .map(|mut u| {
                            u.set_scheme("https").ok();
                            u
                        })
                        .collect::<Vec<_>>(),
                    Kind::Metadata => e
                        .tags
                        .into_iter()
                        .filter_map(|t| match t {
                            Tag::Relay(url) => Url::try_from(url).ok(),
                            _ => None,
                        })
                        .collect::<Vec<_>>(),
                    _ => vec![],
                };

                for u in urls {
                    match self.relay_repo.lock().await.get(u).await {
                        Ok(_) => debug!("successfully saved relay information"),
                        Err(e) => error!("couldn't save relay info: {}", e),
                    };
                }
            })
            .await)
    }
}

type RelayContainer = Arc<Mutex<RelayRepository>>;

pub struct RelayRepository {
    cache: TimedSizedCache<Url, Relay>,
    pool: DbPool,
    http_client: HttpClient,
}

impl RelayRepository {
    pub fn new(pool: DbPool) -> Result<RelayContainer> {
        // lifespan is 6 hours
        let cache = TimedSizedCache::with_size_and_lifespan(500, 6 * 60 * 60);

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, "application/nostr+json".parse().unwrap());

        let http_client = HttpClient::builder().default_headers(headers).build()?;

        Ok(Arc::new(Mutex::new(Self {
            cache,
            pool,
            http_client,
        })))
    }

    pub async fn get(&mut self, url: Url) -> Result<Relay> {
        // check if in cache
        if let Some(relay) = self.cache.cache_get(&url) {
            Ok(relay.clone())
        } else if let Ok(relay) = self.pool.get_relay(&url).await {
            // if in db, check updated is within 12 hours. put in cache and return
            if relay.updated_at.assume_utc()
                > OffsetDateTime::now_utc() - Duration::from_secs(12 * 60 * 60)
            {
                self.cache.cache_set(url, relay.clone());
                Ok(relay)
            } else {
                let relay = self.get_relay_metadata(url.clone()).await?;
                self.pool.update_relay(&relay).await?;
                self.cache.cache_set(url, relay.clone());
                Ok(relay)
            }
        } else {
            let relay = self.get_relay_metadata(url.clone()).await?;
            self.pool.save_relay(&relay).await?;
            self.cache.cache_set(url, relay.clone());
            Ok(relay)
        }
    }

    async fn get_relay_metadata(&self, url: Url) -> Result<Relay> {
        Ok(self
            .http_client
            .get(url.clone())
            .send()
            .await?
            .json::<Relay>()
            .await
            .map(|r| Relay {
                url: url.to_string(),
                seen: true,
                ..r
            })
            .unwrap_or(Relay {
                url: url.to_string(),
                ..Default::default()
            }))
    }
}
