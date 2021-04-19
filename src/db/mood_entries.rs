pub struct MoodEntry {
    id: i32,
    field: i32,
    low: i32,
    entry: i32,
    high: Option<i32>,
    comment: Option<String>
}

impl MoodEntry {

    pub fn getID(&self) -> i32 {
        self.id
    }

    pub fn getField(&self) -> i32 {
        self.field
    }

    pub fn getLow(&self) -> i32 {
        self.low
    }

    pub fn getEntry(&self) -> i32 {
        self.entry
    }

    pub fn getHigh(&self) -> Option<i32> {
        self.high
    }

    pub fn getComment(&self) -> Option<String> {
        match &self.comment {
            Some(com) => Some(com.clone()),
            None => None
        }
    }
}