use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::{Duration, Instant};

use async_std::sync::{Arc, RwLock, RwLockReadGuard};

use rbatis_core::db_adapter::DBPool;

use crate::core::db_adapter::DBTx;
use crate::core::sync::sync_map::{RefMut, SyncMap};
use crate::rbatis::Rbatis;

pub struct TxManager {
    pub tx_context: SyncMap<String, (DBTx, TxState)>,
    pub tx_out_of_time: Duration,
    pub check_interval: Duration,
    pub alive: RwLock<bool>,
}

impl Drop for TxManager {
    fn drop(&mut self) {
        println!("drop");
    }
}


pub enum TxState {
    StateBegin(Instant),
    StateFinish(Instant),
}


impl TxManager {
    pub fn new() -> Self {
        Self {
            tx_context: SyncMap::new(),
            tx_out_of_time: Duration::from_secs(10),
            check_interval: Duration::from_secs(5),
            alive: RwLock::new(true),
        }
    }

    pub async fn set_alive(&self, alive: bool) {
        let mut l = self.alive.write().await;
        *l = alive;
    }
    pub async fn get_alive(&self) -> RwLockReadGuard<'_, bool> {
        self.alive.read().await
    }


    ///polling check tx alive
    pub fn tx_polling_check(manager: &Arc<TxManager>) {
        let manager = manager.clone();
        async_std::task::spawn(async move {
            loop {
                if manager.get_alive().await.deref() == &false {
                    //rollback all
                    let m = manager.tx_context.read().await;
                    let mut rollback_ids =vec![];
                    for (k, (tx, state)) in m.deref() {
                        rollback_ids.push(k.to_string());
                    }
                    drop(m);
                    for tx_id in &rollback_ids {
                        println!("[rbatis] rollback tx_id:{},Because the manager exits", tx_id);
                        manager.rollback(tx_id).await;
                    }
                    drop(manager);
                    println!("tx_polling_check exit;");
                    return;
                }
                let m = manager.tx_context.read().await;
                let mut need_rollback = None;
                for (k, (tx, state)) in m.deref() {
                    match state {
                        TxState::StateBegin(instant) => {
                            let out_time = instant.elapsed();
                            if out_time > manager.tx_out_of_time {
                                if need_rollback == None {
                                    need_rollback = Some(vec![]);
                                }
                                match &mut need_rollback {
                                    Some(v) => {
                                        v.push(k.to_string());
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
                drop(m);
                match &mut need_rollback {
                    Some(v) => {
                        for tx_id in v {
                            println!("[rbatis] rollback tx_id:{},out of time:{:?}", tx_id, &manager.tx_out_of_time);
                            manager.rollback(tx_id).await;
                        }
                        //shrink_to_fit
                        manager.tx_context.shrink_to_fit().await;
                    }
                    _ => {}
                }
                crate::core::runtime::sleep(manager.check_interval).await;
            }
        });
    }


    pub async fn get_mut<'a>(&'a self, tx_id: &str) -> Option<RefMut<'a, String, (DBTx, TxState)>> {
        self.tx_context.get_mut(tx_id).await
    }

    /// begin tx,for new conn
    pub async fn begin(&self, new_tx_id: &str, pool: &DBPool) -> Result<u64, crate::core::Error> {
        if new_tx_id.is_empty() {
            return Err(crate::core::Error::from("[rbatis] tx_id can not be empty"));
        }
        let conn: DBTx = pool.begin().await?;
        //send tx to context
        self.tx_context.insert(new_tx_id.to_string(), (conn, TxState::StateBegin(Instant::now()))).await;
        return Ok(1);
    }

    /// commit tx,and return conn
    pub async fn commit(&self, tx_id: &str) -> Result<u64, crate::core::Error> {
        let tx_op = self.tx_context.remove(tx_id).await;
        if tx_op.is_none() {
            return Err(crate::core::Error::from(format!("[rbatis] tx:{} not exist！", tx_id)));
        }
        let (mut tx, state): (DBTx, TxState) = tx_op.unwrap();
        let result = tx.commit().await?;
        return Ok(1);
    }

    /// rollback tx,and return conn
    pub async fn rollback(&self, tx_id: &str) -> Result<u64, crate::core::Error> {
        let tx_op = self.tx_context.remove(tx_id).await;
        if tx_op.is_none() {
            return Err(crate::core::Error::from(format!("[rbatis] tx:{} not exist！", tx_id)));
        }
        let (tx, state): (DBTx, TxState) = tx_op.unwrap();
        let result = tx.rollback().await?;
        return Ok(1);
    }
}