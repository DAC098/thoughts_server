pub mod params {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct GroupPath {
        pub group_id: i32
    }

    #[derive(Deserialize)]
    pub struct UserPath {
        pub user_id: i32
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
}