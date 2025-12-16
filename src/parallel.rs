#![cfg(not_wasm)]

use anyhow::Result;
use tokio::{spawn, sync::mpsc::channel};

pub async fn first_ok<F, Output>(futures: impl IntoIterator<Item = F>) -> Result<Output>
where
    Output: Send + 'static,
    F: Future<Output = Result<Output>> + Send + 'static, {
    let (s, mut r) = channel::<Output>(1);

    for fut in futures {
        let s = s.clone();
        spawn(async move {
            let result = fut.await;

            if let Ok(result) = result {
                _ = s.send(result).await;
            }
        });
    }

    let result = r.recv().await.unwrap();

    Ok(result)
}

#[cfg(test)]
mod test {
    use anyhow::{anyhow, bail};
    use fake::{Fake, Faker};

    use super::*;

    #[tokio::test]
    async fn all_ok() -> Result<()> {
        let result = first_ok((0..5).map(|_| async move { Ok(55) })).await?;

        assert_eq!(55, result);

        Ok(())
    }

    #[tokio::test]
    async fn some_ok() -> Result<()> {
        let result = first_ok((0..50).map(|_| async move {
            if Faker.fake::<bool>() {
                Ok(77)
            } else {
                bail!("allal")
            }
        }))
        .await?;

        assert_eq!(77, result);

        Ok(())
    }

    #[tokio::test]
    async fn all_err() -> Result<()> {
        let result = first_ok((0..50).map(|_| async move { bail!("allal") })).await?;

        dbg!(&result);

        // assert_eq!(77, result);

        Ok(())
    }
}
