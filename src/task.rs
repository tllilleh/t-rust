use serde::{Deserialize, Serialize};
use sha1::{Sha1, Digest};

#[derive(Serialize, Deserialize)]
pub struct Task {
    id : String,
    #[serde(default)]
    #[serde(skip_serializing)]
    desc : String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_id : Option<String>,
}

impl Task {
    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn desc(&self) -> &String {
        &self.desc
    }

    pub fn to_string(&self) -> String {
        let json = serde_json::to_string(&self).unwrap();
        format!("{} | {}", self.desc(), json)
    }
}

fn split_once(in_string: &str) -> (Option<&str>, Option<&str>) {
    let mut splitter = in_string.splitn(2, '|');
    let first = splitter.next();
    let second = splitter.next();
    (first, second)
}

pub fn create_from_string(string: &str) -> Task {
    let (desc, json) = split_once(string);

    if let Some(desc) = desc {
        //println!("desc: {}", desc);
        if let Some(json) = json {
            //println!("json: {}", json);

            if json.trim().len() == 0 {
                return create(desc);
            }

            let mut task:Task = serde_json::from_str(json).unwrap();
            task.desc = desc.trim().to_string();
            return task
        } else {
            return create(desc);
        }
    }

    // TODO: this should return None or an error?
    return create("");
}

pub fn create(desc: &str) -> Task {
    // create a Sha1 object
    let mut hasher = Sha1::new();
    hasher.update(desc.to_string());
    let id = format!("{:x}", hasher.finalize());

    Task {
        id,
        desc : desc.to_string(),
        parent_id: None
    }
}
