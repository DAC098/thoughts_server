pub mod schema {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    use crate::db::tables::custom_field_entries::CustomFieldEntryType;

    /// full data for an entry marker
    #[derive(Serialize)]
    pub struct Marker {
        pub id: i32,
        pub title: String,
        pub comment: Option<String>,
    }

    /// full data for a custom field entry
    #[derive(Serialize)]
    pub struct CustomField {
        pub field: i32,
        pub value: CustomFieldEntryType,
        pub comment: Option<String>,
    }

    /// full data for a text entry
    #[derive(Serialize)]
    pub struct Text {
        pub id: i32,
        pub thought: String,
        pub private: bool,
    }

    /// full data for an audio entry
    #[derive(Serialize)]
    pub struct Audio {
        pub id: i32,
        pub private: bool,
    }

    /// full data for an entry
    #[derive(Serialize)]
    pub struct Entry {
        pub id: i32,
        pub day: DateTime<Utc>,
        pub created: DateTime<Utc>,
        pub updated: Option<DateTime<Utc>>,
        pub deleted: Option<DateTime<Utc>>,
        pub owner: i32,
        pub tags: Vec<i32>,
        pub markers: Vec<Marker>,
        pub fields: Vec<CustomField>,
        pub text: Vec<Text>,
        pub audio: Vec<Audio>,
    }

    /// partial data for list custom field entry
    #[derive(Serialize)]
    pub struct ListCustomField {
        pub field: i32,
        pub value: CustomFieldEntryType,
    }

    /// partial data for list entry marker
    #[derive(Serialize)]
    pub struct ListMarker {
        pub id: i32,
        pub title: String,
    }

    /// partial data for list entry
    #[derive(Serialize)]
    pub struct ListEntry {
        pub id: i32,
        pub day: DateTime<Utc>,
        pub created: DateTime<Utc>,
        pub updated: Option<DateTime<Utc>>,
        pub deleted: Option<DateTime<Utc>>,
        pub owner: i32,
        pub tags: Vec<i32>,
        pub markers: Vec<ListMarker>,
        pub fields: Vec<ListCustomField>,
        pub text: i64,
        pub audio: i64,
        pub video: i64,
        pub files: i64,
    }

}
