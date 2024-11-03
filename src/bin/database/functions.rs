use diesel::{define_sql_function, sql_types::*};

define_sql_function!(fn f_unaccent(x: Text) -> Text);
