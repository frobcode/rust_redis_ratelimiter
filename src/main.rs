extern crate redis;

use std::fmt;
use std::str::FromStr;

use redis::PipelineCommands;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::time::{Duration, SystemTime};

fn main() -> redis::RedisResult<()> {
    let mything = MinuteTimeMeasurer::new(60);
    do_redis_thing("frobbo", &mything)

}

pub trait TimeMeasurer {
  type Value: fmt::Display + fmt::Debug + FromStr;

  fn get_current_value(&self)->String;
  fn is_active(&self, test_value: &String) -> bool;
  fn expiry_in_duration(&self)->Duration;
}

struct MinuteTimeMeasurer {
  active_window_seconds: u64,
}

impl MinuteTimeMeasurer {
  fn new(window: u64) -> Self {
    MinuteTimeMeasurer {active_window_seconds: window}
  }
}


impl TimeMeasurer for MinuteTimeMeasurer {
  type Value = String;

  fn get_current_value(&self) -> String {
    SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap_or(Duration::new(0, 0))
      .as_secs().to_string()
  }

  fn is_active(&self, test_value: &String) -> bool {
    let top_value: u64 = self.get_current_value().parse().unwrap_or(0);
    let test: u64 = test_value.parse().unwrap_or(0);
    let diff = top_value - test;
    if diff > self.active_window_seconds {
      return false;
    }
    return true;
  }

  fn expiry_in_duration(&self)->Duration {
    return Duration::new(self.active_window_seconds, 0);
  }
}

// let m = RateLimiter::new(accesses_per_minute)
// if m.allow("some_val") {
//   we are ok to allow it in
// else {
//   we probably return a 429
// }
//struct Rate_limiter
//{
//     delete_seconds: u64;
//	: u64;
		
	
fn do_redis_thing(key_name: &str, measurer: &impl TimeMeasurer) -> redis::RedisResult<()> {
	let client = redis::Client::open("redis://127.0.0.1/")?;
	let mut con = client.get_connection()?;
	let hash_key = measurer.get_current_value();
	println!("current hash key is {:?}", hash_key);
	let (getall, ttl, hdel): (BTreeMap<String, i64>, i32, i32) = redis::pipe()
		.hincr(key_name, hash_key, 1).ignore()
		.expire(key_name, 1200).ignore()
		.hgetall(key_name)
		.ttl(key_name)
		.hdel(key_name, vec!["blargh", "fllfj"])
		.query(&mut con)?;
	println!("Got a result!  Expire is {:?}, hdel is {:?}, Hash at {:?} is  {:#?}", ttl, hdel, key_name, getall);
	let mut to_delete = Vec::new();
	let mut total_accesses = 0i64;
	for (k, v) in getall.iter() {
		if measurer.is_active(k) {
			total_accesses += v;
                }
		else {
			to_delete.push(k);
                }
	}
	if to_delete.len() > 0 {
		let to_delete_str = format!("{:#?}", to_delete);
		let (del_res,) : (i32,) = redis::pipe().hdel(key_name, to_delete).query(& mut con)?;
		println!("Keys to delete (too old!): {:?}, result: {:#?}", to_delete_str, del_res);
	}
	println!("Total accesses in the last {:?} seconds: {:?}", measurer.expiry_in_duration(), total_accesses);
	Ok(())
}
