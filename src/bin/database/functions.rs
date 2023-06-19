use diesel::{sql_function, sql_types::*};

sql_function!(fn f_unaccent(x: Text) -> Text);
