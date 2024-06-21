//! Calendar events (iCal `VEVENT` items)

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::item::SyncStatus;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventTime {
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
}

impl EventTime {
    pub fn as_date(&self) -> Option<&NaiveDate> {
        match self {
            EventTime::Date(date) => Some(date),
            _ => None,
        }
    }

    pub fn as_datetime(&self) -> Option<&DateTime<Utc>> {
        match self {
            EventTime::DateTime(datetime) => Some(datetime),
            _ => None,
        }
    }
}

/// TODO: implement `Event` one day.
/// This crate currently only supports tasks, not calendar events.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    uid: String,
    name: String,
    dtstart: EventTime,
    dtend: EventTime,
    location: Option<String>,
    description: Option<String>,
    sync_status: SyncStatus,
    last_modified: DateTime<Utc>,
    creation_date: Option<DateTime<Utc>>,
    ical_prod_id: String,
    url: Url,
}

impl Event {
    pub fn new_timed(
        name: String,
        uid: String,
        dtstart: DateTime<Utc>,
        dtend: DateTime<Utc>,
        location: Option<String>,
        description: Option<String>,
        url: Url,
        sync_status: SyncStatus,
        last_modified: DateTime<Utc>,
        creation_date: Option<DateTime<Utc>>,
        ical_prod_id: String,
    ) -> Self {
        Self {
            name,
            uid,
            dtstart: EventTime::DateTime(dtstart),
            dtend: EventTime::DateTime(dtend),
            location,
            description,
            sync_status,
            last_modified,
            creation_date,
            ical_prod_id,
            url,
        }
    }

    pub fn new_all_day(
        name: String,
        uid: String,
        dtstart: NaiveDate,
        dtend: NaiveDate,
        location: Option<String>,
        description: Option<String>,
        url: Url,
        sync_status: SyncStatus,
        last_modified: DateTime<Utc>,
        creation_date: Option<DateTime<Utc>>,
        ical_prod_id: String,
    ) -> Self {
        Self {
            name,
            uid,
            dtstart: EventTime::Date(dtstart),
            dtend: EventTime::Date(dtend),
            location,
            description,
            sync_status,
            last_modified,
            creation_date,
            ical_prod_id,
            url,
        }
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn uid(&self) -> &str {
        &self.uid
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn dtstart(&self) -> &EventTime {
        &self.dtstart
    }

    pub fn dtend(&self) -> &EventTime {
        &self.dtend
    }

    pub fn location(&self) -> Option<&String> {
        self.location.as_ref()
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn ical_prod_id(&self) -> &str {
        &self.ical_prod_id
    }

    pub fn last_modified(&self) -> &DateTime<Utc> {
        &self.last_modified
    }

    pub fn creation_date(&self) -> Option<&DateTime<Utc>> {
        self.creation_date.as_ref()
    }

    pub fn sync_status(&self) -> &SyncStatus {
        &self.sync_status
    }

    pub fn set_sync_status(&mut self, new_status: SyncStatus) {
        self.sync_status = new_status;
    }

    #[cfg(any(test, feature = "integration_tests"))]
    pub fn has_same_observable_content_as(&self, other: &Event) -> bool {
        self.uid == other.uid
            && self.name == other.name
            && self.dtstart == other.dtstart
            && self.dtend == other.dtend
            && self.location == other.location
            && self.description == other.description
            && self.last_modified == other.last_modified
            && self.ical_prod_id == other.ical_prod_id
    }
}
