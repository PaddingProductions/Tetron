macro_rules! dev_log {
    ($s:literal) => {
        if std::env::var("LOG").is_ok() {
            print!($s);
        }
    };
    (ln, $s:literal) => {
        if std::env::var("LOG").is_ok() {
            println!($s);
        }
    };
    ($s:literal, $($a: expr),* ) => {
        if std::env::var("LOG").is_ok() {
            print!(
                $s,
                $($a,)*
            );
        }
    };
    (ln, $s:literal, $($a: expr),* ) => {
        if std::env::var("LOG").is_ok() {
            println!(
                $s,
                $($a,)*
            );
        }
    };
}
pub(crate) use dev_log;