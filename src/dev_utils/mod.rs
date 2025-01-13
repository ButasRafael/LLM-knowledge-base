mod dev_db;

use tokio::sync::OnceCell;

pub async fn init_dev(){
    static INIT: OnceCell<()> = OnceCell::const_new();
    INIT.get_or_init(|| async {
        println!("->> {:<12} - init_dev.all()", "FOR-DEV-ONLY");
        dev_db::init_dev_db().await.unwrap();
    }).await;
}