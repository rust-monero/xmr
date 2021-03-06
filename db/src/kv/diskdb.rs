// Xmr, Monero node.
// Copyright (C) 2018  Jean Pierre Dudey
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::fmt::{self, Debug, Formatter};
use std::path::Path;
use std::cmp::max;

use rand::OsRng;
use sanakirja::{Env, Error, MutTxn, Commit, Db, Transaction as SanakirjaTransaction};
use sanakirja::value::UnsafeValue;

use kv::{KeyValueDatabase, KeyState, Key, Value, Transaction};
use kv::transaction::{RawOperation, RawKey};

/// A database stored in disk.
pub struct DiskDb {
    /// Sanakirja environment.
    env: Env,
}

impl DiskDb {
    /// Open a database.
    ///
    /// It takes a path to a directory, not a file.
    pub fn open<P>(path: P) -> Result<DiskDb, Error>
        where P: AsRef<Path>
    {
        let size = Self::db_size(path.as_ref());
        let env = Env::new::<P>(path, size)?;

        Ok(DiskDb { env: env })
    }

    /// Query the database file size.
    fn db_size<P>(path: P) -> u64
        where P: AsRef<Path>
    {
        // XXX: Is this the best default?
        const MIN_DB_SIZE: u64 = 1 << 17;

        Env::file_size(path.as_ref())
            .map(|size| max(size, MIN_DB_SIZE))
            .unwrap_or(MIN_DB_SIZE)
    }
}

impl KeyValueDatabase for DiskDb {
    // TODO: Unwraps to errors.
    fn write(&self, tx: Transaction) -> Result<(), String> {
        let mut txn = self.env.mut_txn_begin().unwrap();
        let mut prng = OsRng::new().unwrap();

        // XXX: probably not the best performant kv db out there, but... who cares?
        for op in tx.operations.iter() {
            let op = op.into();
            match op {
                RawOperation::Insert(ref kv) => {
                    let mut db = open_db(&mut txn, kv.location);
                    let k = UnsafeValue::from_slice(kv.key.as_ref());
                    let v = UnsafeValue::from_slice(kv.value.as_ref());
                    txn.put::<_, _, UnsafeValue>(&mut prng, &mut db, k, v)
                        .unwrap();
                    txn.set_root(kv.location, db);
                }
                RawOperation::Delete(ref k) => {
                    let mut db = open_db(&mut txn, k.location);
                    let key = UnsafeValue::from_slice(k.key.as_ref());
                    txn.del::<_, _, UnsafeValue>(&mut prng, &mut db, key, None)
                        .unwrap();
                    txn.set_root(k.location, db);
                }
            }
        }

        txn.commit().unwrap();

        Ok(())
    }

    fn get(&self, key: &Key) -> Result<KeyState<Value>, String> {
        let raw_key: RawKey = key.into();
        let mut txn = self.env.txn_begin().unwrap();
        let db = match txn.root(raw_key.location) {
            Some(db) => db,
            None => return Ok(KeyState::Unknown),
        };

        let key_val = UnsafeValue::from_slice(&raw_key.key);
        let val = txn.get::<_, UnsafeValue>(&db, key_val, None)
            .ok_or("key doesn't exists".to_owned());
        if let Ok(val) = val {
            Ok(KeyState::Insert(Value::for_key(key, unsafe { val.as_slice() })))
        } else {
            Ok(KeyState::Delete)
        }
    }
}

fn open_db(txn: &mut MutTxn<()>, root: usize) -> Db<UnsafeValue, UnsafeValue> {
    if let Some(db) = txn.root(root) {
        db
    } else {
        // TODO: no unwrap
        txn.create_db().unwrap()
    }
}

impl Debug for DiskDb {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        write!(fmt, "DiskDb")
    }
}

#[cfg(test)]
pub mod tests {
    extern crate tempdir;

    use self::tempdir::TempDir;

    use super::*;
    use super::super::*;

    #[test]
    fn test_db() {
        let hash = [0x01, 0x3c, 0x01, 0xff, 0x00, 0x01, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x03,
                    0x02, 0x9b, 0x2e, 0x4c, 0x02, 0x81, 0xc0, 0xb0, 0x2e, 0x7c, 0x53, 0x29, 0x1a,
                    0x94, 0xd1, 0xd0, 0xcb, 0xff, 0x88];

        let tempdir = TempDir::new("").unwrap();
        let db = DiskDb::open(tempdir.path()).unwrap();

        let mut tx = Transaction::new();

        let kv = KeyValue::BlockHeight(hash.into(), 0);
        tx.insert(kv);

        db.write(tx).unwrap();

        let k = Key::BlockHeight(hash.into());
        match db.get(&k).unwrap() {
            KeyState::Insert(Value::BlockHeight(0)) => { /* happy path */ }
            KeyState::Insert(_) => panic!("invalid value"),
            KeyState::Delete => panic!("key-value pair is deleted"),
            KeyState::Unknown => panic!("key-value pair is unknown"),
        }
    }
}
