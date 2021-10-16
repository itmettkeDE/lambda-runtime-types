#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct Secret {
    host: String,
    port: u16,
    database: String,
    user: String,
    password: String,
}

struct Runner;

#[async_trait::async_trait]
impl lambda_runtime_types::rotate::RotateRunner<(), Secret> for Runner {
    async fn setup() -> anyhow::Result<()> {
        simple_logger::SimpleLogger::new()
            .with_level(log::LevelFilter::Info)
            .init()
            .expect("Unable to setup logging");
        Ok(())
    }

    async fn create(
        _shared: &(),
        mut secret_cur: lambda_runtime_types::rotate::SecretContainer<Secret>,
        smc: &lambda_runtime_types::rotate::Smc,
        _region: &str,
    ) -> anyhow::Result<lambda_runtime_types::rotate::SecretContainer<Secret>> {
        let password = smc.generate_new_password(false, None).await?;
        secret_cur.password = password;
        Ok(secret_cur)
    }

    async fn set(
        _shared: &(),
        secret_cur: lambda_runtime_types::rotate::SecretContainer<Secret>,
        secret_new: lambda_runtime_types::rotate::SecretContainer<Secret>,
        _region: &str,
    ) -> anyhow::Result<()> {
        PgDatabase::new(&secret_cur)
            .await?
            .change_password(&secret_new)
            .await
    }

    async fn test(
        _shared: &(),
        secret_new: lambda_runtime_types::rotate::SecretContainer<Secret>,
        _region: &str,
    ) -> anyhow::Result<()> {
        PgDatabase::new(&secret_new)
            .await?
            .test_connection()
            .await
    }
}

pub fn main() -> anyhow::Result<()> {
    lambda_runtime_types::exec_tokio::<_, _, Runner, _>()
}

pub struct PgDatabase {
    client: tokio_postgres::Client,
}

impl PgDatabase {
    pub(crate) async fn new(secret: &Secret) -> anyhow::Result<Self> {
        use anyhow::Context;

        let connector = native_tls::TlsConnector::new()
            .context("Unable to prepare TLS Connection for Database")?;
        let connector = postgres_native_tls::MakeTlsConnector::new(connector);

        let (client, connection) = tokio_postgres::connect(
            &format!(
                "host={host} port={port} user={user} password={password} dbname={dbname} sslmode=require",
                host = secret.host,
                port = secret.port,
                user = secret.user,
                password = secret.password,
                dbname = secret.database,
            ),
            connector,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Unable to connecto to postgres database: {}", e);
            }
        });

        Ok(Self { client })
    }

    pub(crate) async fn change_password(&self, secret: &Secret) -> anyhow::Result<()> {
        use anyhow::Context;

        let query: &str = &format!(
            "ALTER USER {} WITH PASSWORD '{}'",
            secret.user, secret.password
        );
        self.client
            .execute(query, &[])
            .await
            .context("Unable to change user password")?;
        Ok(())
    }

    pub(crate) async fn test_connection(&self) -> anyhow::Result<()> {
        use anyhow::Context;

        self.client
            .execute("SELECT 1;", &[])
            .await
            .context("Connection to database failed")?;
        Ok(())
    }
}
