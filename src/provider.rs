//! This modules abstracts data sources and merges them in a single virtual one

use std::error::Error;
use std::collections::{HashSet, HashMap};
use std::marker::PhantomData;

use chrono::{DateTime, Utc};

use crate::traits::{CalDavSource, CompleteCalendar};
use crate::traits::SyncSlave;
use crate::traits::PartialCalendar;
use crate::Item;
use crate::item::ItemId;


/// A data source that combines two `CalDavSources` (usually a server and a local cache), which is able to sync both sources.
pub struct Provider<L, T, S, U>
where
    L: CalDavSource<T> + SyncSlave,
    T: CompleteCalendar,
    S: CalDavSource<U>,
    U: PartialCalendar,
{
    /// The remote server
    server: S,
    /// The local cache
    local: L,

    phantom_t: PhantomData<T>,
    phantom_u: PhantomData<U>,
}

impl<L, T, S, U> Provider<L, T, S, U>
where
    L: CalDavSource<T> + SyncSlave,
    T: CompleteCalendar,
    S: CalDavSource<U>,
    U: PartialCalendar,
{
    /// Create a provider.
    ///
    /// `server` is usually a [`Client`](crate::client::Client), `local` is usually a [`Cache`](crate::cache::Cache).
    /// However, both can be interchangeable. The only difference is that `server` always wins in case of a sync conflict
    pub fn new(server: S, local: L) -> Self {
        Self { server, local,
            phantom_t: PhantomData, phantom_u: PhantomData,
        }
    }

    /// Returns the data source described as the `server`
    pub fn server(&self) -> &S { &self.server }
    /// Returns the data source described as the `local`
    pub fn local(&self)  -> &L { &self.local }
    /// Returns the last time the `local` source has been synced
    pub fn last_sync_timestamp(&self) -> Option<DateTime<Utc>> {
        self.local.get_last_sync()
    }

    /// Performs a synchronisation between `local` and `server`.
    ///
    /// This bidirectional sync applies additions/deleteions made on a source to the other source.
    /// In case of conflicts (the same item has been modified on both ends since the last sync, `server` always wins)
    pub async fn sync(&mut self) -> Result<(), Box<dyn Error>> {
        let last_sync = self.local.get_last_sync();
        log::info!("Starting a sync. Last sync was at {:?}", last_sync);
        let cals_server = self.server.get_calendars().await?;

        for (id, mut cal_server) in cals_server {
            let mut cal_server = cal_server.lock().unwrap();

            let cal_local = match self.local.get_calendar(id).await {
                None => {
                    log::error!("TODO: implement here");
                    continue;
                },
                Some(cal) => cal,
            };
            let mut cal_local = cal_local.lock().unwrap();

            // Pull remote changes from the server
            let mut tasks_id_to_remove_from_local = match last_sync {
                None => Vec::new(),
                Some(_date) => cal_server.find_deletions_from(cal_local.get_item_ids())
                    .iter()
                    .map(|id| id.clone())
                    .collect()
            };

            let mut tasks_to_add_to_local = Vec::new();
            let server_mod = cal_server.get_items_modified_since(last_sync, None);
            for (new_id, new_item) in &server_mod {
                if server_mod.contains_key(new_id) {
                    log::warn!("Conflict for task {} ({}). Using the server version.", new_item.name(), new_id);
                    tasks_id_to_remove_from_local.push(new_id.clone());
                }
                tasks_to_add_to_local.push((*new_item).clone());
            }
            // Even in case of conflicts, "the server always wins", so it is safe to remove tasks from the local cache as soon as now
            remove_from_calendar(&tasks_id_to_remove_from_local, &mut *cal_local);



            // Push local changes to the server
            let local_del = match last_sync {
                Some(date) => cal_local.get_items_deleted_since(date),
                None => HashSet::new(),
            };
            let mut tasks_id_to_remove_from_server = Vec::new();
            for deleted_id in local_del {
                if server_mod.contains_key(&deleted_id) {
                    log::warn!("Conflict for task {}, that has been locally deleted and updated in the server. Using the server version.", deleted_id);
                    continue;
                }
                tasks_id_to_remove_from_server.push(deleted_id);
            }

            let local_mod = cal_local.get_items_modified_since(last_sync, None);
            let mut tasks_to_add_to_server = Vec::new();
            for (new_id, new_item) in &local_mod {
                if server_mod.contains_key(new_id) {
                    log::warn!("Conflict for task {} ({}). Using the server version.", new_item.name(), new_id);
                    continue;
                }
                tasks_to_add_to_server.push((*new_item).clone());
            }

            remove_from_calendar(&tasks_id_to_remove_from_server, &mut *cal_server);
            move_to_calendar(&mut tasks_to_add_to_local, &mut *cal_local);
            move_to_calendar(&mut tasks_to_add_to_server, &mut *cal_server);
        }

        self.local.update_last_sync(None);

        Ok(())
    }
}


fn move_to_calendar<C: PartialCalendar>(items: &mut Vec<Item>, calendar: &mut C) {
    while items.len() > 0 {
        let item = items.remove(0);
        calendar.add_item(item);
    }
}

fn remove_from_calendar<C: PartialCalendar>(ids: &Vec<ItemId>, calendar: &mut C) {
    for id in ids {
        log::info!("  Removing {:?} from local calendar", id);
        calendar.delete_item(id);
    }
}
