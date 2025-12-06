use crate::camera_stream::CameraStream;
use anyhow::{Result, anyhow};
use opencv::core::{Mat, MatTraitConstManual};
use opencv::prelude::MatTraitConst;
use std::collections::BTreeMap;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub struct Workloads {
    pub camera_streams: BTreeMap<String, CameraStream>,
}
