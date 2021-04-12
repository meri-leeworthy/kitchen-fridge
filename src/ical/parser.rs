//! A module to parse ICal files

use std::error::Error;

use ical::parser::ical::component::{IcalCalendar, IcalEvent, IcalTodo};

use crate::Item;
use crate::item::SyncStatus;
use crate::item::ItemId;
use crate::Task;
use crate::Event;


/// Parse an iCal file into the internal representation [`crate::Item`]
pub fn parse(content: &str, item_id: ItemId, sync_status: SyncStatus) -> Result<Item, Box<dyn Error>> {
    let mut reader = ical::IcalParser::new(content.as_bytes());
    let parsed_item = match reader.next() {
        None => return Err(format!("Invalid uCal data to parse for item {}", item_id).into()),
        Some(item) => match item {
            Err(err) => return Err(format!("Unable to parse uCal data for item {}: {}", item_id, err).into()),
            Ok(item) => item,
        }
    };

    let item = match assert_single_type(&parsed_item)? {
        CurrentType::Event(_) => {
            Item::Event(Event::new())
        },

        CurrentType::Todo(todo) => {
            let mut name = None;
            let mut uid = None;
            let mut completed = false;
            for prop in &todo.properties {
                if prop.name == "SUMMARY" {
                    name = prop.value.clone();
                }
                if prop.name == "STATUS" {
                    // Possible values:
                    //   "NEEDS-ACTION" ;Indicates to-do needs action.
                    //   "COMPLETED"    ;Indicates to-do completed.
                    //   "IN-PROCESS"   ;Indicates to-do in process of.
                    //   "CANCELLED"    ;Indicates to-do was cancelled.
                    if prop.value.as_ref().map(|s| s.as_str()) == Some("COMPLETED") {
                        completed = true;
                    }
                }
                if prop.name == "UID" {
                    uid = prop.value.clone();
                }
            }
            let name = match name {
                Some(name) => name,
                None => return Err(format!("Missing name for item {}", item_id).into()),
            };
            let uid = match uid {
                Some(uid) => uid,
                None => return Err(format!("Missing UID for item {}", item_id).into()),
            };

            Item::Task(Task::new_with_parameters(name, completed, uid, item_id, sync_status))
        },
    };


    // What to do with multiple items?
    if reader.next().map(|r| r.is_ok()) == Some(true) {
        return Err("Parsing multiple items are not supported".into());
    }

    Ok(item)
}

enum CurrentType<'a> {
    Event(&'a IcalEvent),
    Todo(&'a IcalTodo),
}

fn assert_single_type<'a>(item: &'a IcalCalendar) -> Result<CurrentType<'a>, Box<dyn Error>> {
    let n_events = item.events.len();
    let n_todos = item.todos.len();
    let n_journals = item.journals.len();

    if n_events == 1 {
        if n_todos != 0 || n_journals != 0 {
            return Err("Only a single TODO or a single EVENT is supported".into());
        } else {
            return Ok(CurrentType::Event(&item.events[0]));
        }
    }

    if n_todos == 1 {
        if n_events != 0 || n_journals != 0 {
            return Err("Only a single TODO or a single EVENT is supported".into());
        } else {
            return Ok(CurrentType::Todo(&item.todos[0]));
        }
    }

    return Err("Only a single TODO or a single EVENT is supported".into());
}


#[cfg(test)]
mod test {
    const EXAMPLE_ICAL: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Nextcloud Tasks v0.13.6
BEGIN:VTODO
UID:0633de27-8c32-42be-bcb8-63bc879c6185@some-domain.com
CREATED:20210321T001600
LAST-MODIFIED:20210321T001600
DTSTAMP:20210321T001600
SUMMARY:Do not forget to do this
END:VTODO
END:VCALENDAR
"#;

    const EXAMPLE_ICAL_COMPLETED: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Nextcloud Tasks v0.13.6
BEGIN:VTODO
UID:19960401T080045Z-4000F192713-0052@example.com
CREATED:20210321T001600
LAST-MODIFIED:20210402T081557
DTSTAMP:20210402T081557
SUMMARY:Clean up your room or Mom will be angry
PERCENT-COMPLETE:100
COMPLETED:20210402T081557
STATUS:COMPLETED
END:VTODO
END:VCALENDAR
"#;

    const EXAMPLE_MULTIPLE_ICAL: &str = r#"BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//Nextcloud Tasks v0.13.6
BEGIN:VTODO
UID:0633de27-8c32-42be-bcb8-63bc879c6185
CREATED:20210321T001600
LAST-MODIFIED:20210321T001600
DTSTAMP:20210321T001600
SUMMARY:Call Mom
END:VTODO
END:VCALENDAR
BEGIN:VCALENDAR
BEGIN:VTODO
UID:0633de27-8c32-42be-bcb8-63bc879c6185
CREATED:20210321T001600
LAST-MODIFIED:20210321T001600
DTSTAMP:20210321T001600
SUMMARY:Buy a gift for Mom
END:VTODO
END:VCALENDAR
"#;

    use super::*;
    use crate::item::VersionTag;

    #[test]
    fn test_ical_parsing() {
        let version_tag = VersionTag::from(String::from("test-tag"));
        let sync_status = SyncStatus::Synced(version_tag);
        let item_id: ItemId = "http://some.id/for/testing".parse().unwrap();

        let item = parse(EXAMPLE_ICAL, item_id.clone(), sync_status.clone()).unwrap();
        let task = item.unwrap_task();

        assert_eq!(task.name(), "Do not forget to do this");
        assert_eq!(task.id(), &item_id);
        assert_eq!(task.uid(), "0633de27-8c32-42be-bcb8-63bc879c6185@some-domain.com");
        assert_eq!(task.completed(), false);
        assert_eq!(task.sync_status(), &sync_status);
    }

    #[test]
    fn test_ical_completion_parsing() {
        let version_tag = VersionTag::from(String::from("test-tag"));
        let sync_status = SyncStatus::Synced(version_tag);
        let item_id: ItemId = "http://some.id/for/testing".parse().unwrap();

        let item = parse(EXAMPLE_ICAL_COMPLETED, item_id.clone(), sync_status.clone()).unwrap();
        let task = item.unwrap_task();

        assert_eq!(task.completed(), true);
    }

    #[test]
    fn test_multiple_items_in_ical() {
        let version_tag = VersionTag::from(String::from("test-tag"));
        let sync_status = SyncStatus::Synced(version_tag);
        let item_id: ItemId = "http://some.id/for/testing".parse().unwrap();

        let item = parse(EXAMPLE_MULTIPLE_ICAL, item_id.clone(), sync_status.clone());
        assert!(item.is_err());
    }
}
