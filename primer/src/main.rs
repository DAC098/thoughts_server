fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(run());
    
    println!("primer finished");
}

async fn run() {
    let (mut client, connection) = tokio_postgres::connect(
        "postgresql://postgres:password@truth.dac098.com:5434/thoughts",
        tokio_postgres::NoTls
    )
        .await
        .expect("failed to connect to database");

    let conn_thread = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("database connection error: {}", e);
        } else {
            println!("database connection closed");
        }
    });

    let admin_check = client.query_opt(
        "select * from users where username = 'admin'",
        &[]
    )
        .await
        .expect("failed to query database for admin user");

    if let Some(_record) = admin_check {
        println!("admin record was found");
    } else {
        let hash_config = argon2::Config {
            variant: argon2::Variant::Argon2i,
            version: argon2::Version::Version13,
            mem_cost: 65536,
            time_cost: 10,
            lanes: 4,
            thread_mode: argon2::ThreadMode::Parallel,
            secret: &[],
            ad: &[],
            hash_length: 32
        };
        let mut rand_bytes: Vec<u8> = Vec::with_capacity(64);
        
        for _ in 0..rand_bytes.capacity() {
            rand_bytes.push(0);
        }

        openssl::rand::rand_bytes(rand_bytes.as_mut_slice())
            .expect("failed to generate salt for admin password");

        println!("creating administrative user");

        let username = "admin".to_owned();
        let password = b"password";
        let full_name = None::<String>;
        let email = None::<String>;
        let level = 1;
        let hash = argon2::hash_encoded(password, rand_bytes.as_slice(), &hash_config)
            .expect("failed to generate password hash");

        let transaction = client.transaction()
            .await
            .expect("failed to start database transaction");

        transaction.query_one(
            "\
            insert into users (level, username, full_name, hash, email) values \
            ($1, $2, $3, $4, $5) \
            returning id",
            &[
                &level,
                &username,
                &full_name,
                &hash,
                &email
            ]
        ).await
            .expect("failed to insert admin record into database");
        
        transaction.commit()
            .await
            .expect("failed to commit transaction");
    }

    std::mem::drop(client);

    conn_thread.await
        .unwrap();
}
