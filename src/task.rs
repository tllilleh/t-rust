use serde::{Deserialize, Serialize};
use sha1::{Sha1, Digest};

fn is_false(operand: &bool) -> bool {
    !operand
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    id : String,
    #[serde(default, skip_serializing)]
    desc : String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    parent_id : Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    show_full_id : bool,
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

    pub fn show_full_id(&self) -> bool {
        self.show_full_id
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
                return create(None, desc);
            }

            let mut task:Task = serde_json::from_str(json).unwrap();
            task.desc = desc.trim().to_string();
            return task
        } else {
            return create(None, desc);
        }
    }

    // TODO: this should return None or an error?
    return create(None, "");
}

pub fn create(id_in: Option<&str>, desc: &str) -> Task {
    let id: String;
    let show_full_id: bool;

    match id_in {
        None => {
            // create a Sha1 object
            let mut hasher = Sha1::new();
            hasher.update(desc.to_string());
            id = format!("{:x}", hasher.finalize());
            show_full_id = false;
        },
        Some(user_provided_id) => {
            id = user_provided_id.to_string();
            show_full_id = true;
        }
    }

    Task {
        id,
        desc : desc.to_string(),
        parent_id: None,
        show_full_id,
    }
}
