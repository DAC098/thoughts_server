pub mod schema {
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    use crate::db::tables::{
        custom_field_entries::{CustomFieldEntry, CustomFieldEntryType},
        audio_entries::AudioEntry,
        text_entries::TextEntry,
        entry_markers::EntryMarker,
    };

    /// full data for an entry marker
    #[derive(Serialize)]
    pub struct Marker {
        pub id: i32,
        pub title: String,
        pub comment: Option<String>,
    }

    impl From<EntryMarker> for Marker {
        fn from(v: EntryMarker) -> Marker {
            Marker {
                id: v.id,
                title: v.title,
                comment: v.comment,
            }
        }
    }

    /// full data for a custom field entry
    #[derive(Serialize)]
    pub struct CustomField {
        pub field: i32,
        pub value: CustomFieldEntryType,
        pub comment: Option<String>,
    }

    impl From<CustomFieldEntry> for CustomField {
        fn from(v: CustomFieldEntry) -> CustomField {
            CustomField {
                field: v.field,
                value: v.value,
                comment: v.comment,
            }
        }
    }

    /// full data for a text entry
    #[derive(Serialize)]
    pub struct Text {
        pub id: i32,
        pub thought: String,
        pub private: bool,
    }

    impl From<TextEntry> for Text {
        fn from(v: TextEntry) -> Text {
            Text {
                id: v.id,
                thought: v.thought,
                private: v.private,
            }
        }
    }

    /// full data for an audio entry
    #[derive(Serialize)]
    pub struct Audio {
        pub id: i32,
        pub private: bool,
        pub mime: String,
        pub size: i64,
    }

    impl From<AudioEntry> for Audio {
        fn from(v: AudioEntry) -> Audio {
            Audio {
                id: v.id,
                private: v.private,
                mime: format!("{}/{}", v.mime_type, v.mime_subtype),
                size: v.file_size,
            }
        }
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

    impl From<CustomFieldEntry> for ListCustomField {
        fn from(v: CustomFieldEntry) -> ListCustomField {
            ListCustomField {
                field: v.field,
                value: v.value,
            }
        }
    }

    /// partial data for list entry marker
    #[derive(Serialize)]
    pub struct ListMarker {
        pub id: i32,
        pub title: String,
    }

    impl From<EntryMarker> for ListMarker {
        fn from(v: EntryMarker) -> ListMarker {
            ListMarker {
                id: v.id,
                title: v.title,
            }
        }
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
