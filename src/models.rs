use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::{types::Json, FromRow};
use std::{collections::HashMap, ops::Deref};
use time::{OffsetDateTime, PrimitiveDateTime};

#[derive(Clone)]
pub struct DateTime(PrimitiveDateTime);

#[derive(Deserialize, Clone, Serialize, Default)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Relay {
    #[serde(skip_deserializing)]
    pub url: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub pubkey: Option<String>,
    pub contact: Option<String>,
    #[cfg(feature = "ssr")]
    pub supported_nips: Option<Json<Vec<i32>>>,
    #[cfg(not(feature = "ssr"))]
    pub supported_nips: Option<Vec<i32>>,
    pub software: Option<String>,
    pub version: Option<String>,
    #[cfg(feature = "ssr")]
    pub limitation: Option<Json<Limitation>>,
    #[cfg(not(feature = "ssr"))]
    pub limitation: Option<Limitation>,
    pub retention: Option<String>,
    #[cfg(feature = "ssr")]
    pub relay_countries: Option<Json<Vec<String>>>,
    #[cfg(not(feature = "ssr"))]
    pub relay_countries: Option<Vec<String>>,
    #[cfg(feature = "ssr")]
    pub language_tags: Option<Json<Vec<String>>>,
    #[cfg(not(feature = "ssr"))]
    pub language_tags: Option<Vec<String>>,
    #[cfg(feature = "ssr")]
    pub tags: Option<Json<Vec<String>>>,
    #[cfg(not(feature = "ssr"))]
    pub tags: Option<Vec<String>>,
    pub posting_policy: Option<String>,
    pub payments_url: Option<String>,
    #[cfg(feature = "ssr")]
    pub fees: Option<Json<HashMap<String, Vec<FeeSchedule>>>>,
    #[cfg(not(feature = "ssr"))]
    pub fees: Option<HashMap<String, Vec<FeeSchedule>>>,
    pub icon: Option<String>,
    #[serde(skip_deserializing)]
    pub created_at: DateTime,
    #[serde(skip_deserializing)]
    pub updated_at: DateTime,
    #[serde(skip_deserializing)]
    pub seen: bool,
}

impl Default for DateTime {
    fn default() -> Self {
        let now = OffsetDateTime::now_utc();
        Self(PrimitiveDateTime::new(now.date(), now.time()))
    }
}

impl Deref for DateTime {
    type Target = PrimitiveDateTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<PrimitiveDateTime> for DateTime {
    fn from(value: PrimitiveDateTime) -> Self {
        Self(value)
    }
}

impl Into<PrimitiveDateTime> for DateTime {
    fn into(self) -> PrimitiveDateTime {
        self.0
    }
}

impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(PrimitiveDateTime::deserialize(deserializer)?))
    }
}

#[derive(Deserialize, Clone, Serialize)]
pub struct Limitation {
    pub max_message_length: Option<i32>,
    pub max_subscriptions: Option<i32>,
    pub max_filters: Option<i32>,
    pub max_limit: Option<i32>,
    pub max_subid_length: Option<i32>,
    pub min_prefix: Option<i32>,
    pub max_event_tags: Option<i32>,
    pub max_content_length: Option<i32>,
    pub min_pow_difficulty: Option<i32>,
    pub auth_required: Option<bool>,
    pub payment_required: Option<bool>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct FeeSchedule {
    pub amount: i32,
    pub unit: String,
    pub period: Option<i32>,
    pub kinds: Option<Vec<String>>,
}
