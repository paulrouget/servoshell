/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use log::*;
use std::sync::{Arc, Mutex};

pub struct ShellLog {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
}

pub struct ShellLogs(Mutex<Vec<ShellLog>>);

impl ShellLogs {
    pub fn get_logs(&self) -> Vec<ShellLog> {
        let mut vec = self.0.lock().unwrap();
        let res = vec.drain(..).collect();
        res
    }
}

pub struct Logger(Arc<ShellLogs>);

impl Logger {
    pub fn init() -> Arc<ShellLogs> {
        let mut rv = None;
        set_logger(|max_log_level| {
            max_log_level.set(LogLevelFilter::Info);
            let logs = Arc::new(ShellLogs(Mutex::new(Vec::new())));
            rv = Some(logs.clone());
            Box::new(Logger(logs))
        }).unwrap();
        rv.unwrap()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &LogMetadata) -> bool {
        metadata.level() <= LogLevel::Info
    }

    fn log(&self, record: &LogRecord) {
        if self.enabled(record.metadata()) {
            let log = ShellLog {
                level: record.level(),
                message: format!("{}", record.args()),
                target: format!("{}", record.target()),
            };
            (self.0).0.lock().unwrap().push(log);
        }
    }
}
