use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::time::SystemTime;

fn is_false(operand: &bool) -> bool {
    !operand
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    id: String,
    #[serde(default, skip_serializing)]
    desc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    show_full_id: bool,
    #[serde(default)]
    timestamp: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
}

impl Task {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn desc(&self) -> &String {
        &self.desc
    }

    pub fn to_file_string(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("{} | {}", self.desc(), json)
    }

    pub fn show_full_id(&self) -> bool {
        self.show_full_id
    }

    pub fn timestamp(&self) -> f64 {
        self.timestamp
    }

    pub fn set_desc(&mut self, desc: &str) {
        self.desc = desc.to_string();
    }

    pub fn parent_id(&self) -> &Option<String> {
        &self.parent_id
    }

    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|x| x != tag);
    }

    pub fn tags(&self) -> &Vec<String> {
        &self.tags
    }
}

fn split_once(in_string: &str) -> (Option<&str>, Option<&str>) {
    let mut splitter = in_string.splitn(2, '|');
    let first = splitter.next();
    let second = splitter.next();
    (first, second)
}

pub fn create_from_file_string(string: &str) -> Task {
    let (desc, json) = split_once(string);

    if let Some(desc) = desc {
        //println!("desc: {}", desc);
        if let Some(json) = json {
            //println!("json: {}", json);

            if json.trim().is_empty() {
                return create(None, None, desc);
            }

            let mut task: Task = serde_json::from_str(json).unwrap();
            task.desc = desc.trim().to_string();
            return task;
        } else {
            return create(None, None, desc);
        }
    }

    // TODO: this should return None or an error?
    create(None, None, "")
}

pub fn create(parent_id: Option<&str>, id: Option<&str>, desc: &str) -> Task {
    let show_full_id: bool;
    let timestamp: f64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Err(_) => 0.0,
        Ok(ts) => ts.as_secs_f64(),
    };

    let id = match id {
        None => {
            // create a Sha1 object
            let mut hasher = Sha1::new();
            hasher.update(desc);
            hasher.update(timestamp.to_string());
            show_full_id = false;
            format!("{:x}", hasher.finalize())
        }
        Some(user_provided_id) => {
            show_full_id = true;
            user_provided_id.to_string()
        }
    };

    Task {
        id,
        desc: desc.to_string(),
        parent_id: parent_id.map(String::from),
        show_full_id,
        timestamp,
        tags: Vec::new(),
    }
}
