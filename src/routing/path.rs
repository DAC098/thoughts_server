//! common code for dealing with pathing to handlers

pub mod params {
    //! path params for handlers

    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct GroupPath {
        pub group_id: i32
    }

    /// handleing a given user id
    #[derive(Deserialize)]
    pub struct UserPath {
        pub user_id: i32
    }

    /// potentially handling a given user id
    #[derive(Deserialize)]
    pub struct OptUserPath {
        pub user_id: Option<i32>,
    }

    #[derive(Deserialize)]
    pub struct UserEntryPath {
        pub user_id: i32,
        pub entry_id: i32
    }

    #[derive(Deserialize)]
    pub struct UserFieldPath {
        pub user_id: i32,
        pub field_id: i32
    }

    /// common params for dealing with entries only
    ///
    /// optionally handles a user_id if possible
    #[derive(Deserialize)]
    pub struct EntryPath {
        pub user_id: Option<i32>,
        pub entry_id: i32
    }

    /// common params for dealing with audio entries
    ///
    /// optionally handles user_id if possible
    #[derive(Deserialize)]
    pub struct EntryAudioPath {
        pub user_id: Option<i32>,
        pub entry_id: i32,
        pub audio_id: i32,
    }

    /// common params for dealing with comments for entries
    ///
    /// optionally handles user_id if possible
    #[derive(Deserialize)]
    pub struct EntryCommentPath {
        pub user_id: Option<i32>,
        pub entry_id: i32,
        pub comment_id: i32,
    }

    /// path params for custom fields
    ///
    /// optionally handles user_id if possible
    #[derive(Deserialize)]
    pub struct CustomFieldPath {
        pub user_id: Option<i32>,
        pub field_id: i32,
    }
}
