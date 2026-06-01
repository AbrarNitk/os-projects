use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom, Write},
};

pub struct Database {
    cache: HashMap<String, String>,
    persistent: std::fs::File,
}

impl Database {
    pub fn put(&mut self, key: String, value: String) {
        let row = Row::new_i(key, value);
        self.write_row(&row);
        if let Row::I { key, value } = row {
            self.cache.insert(key, value);
        }
    }

    pub fn keys(&self) {
        for k in self.cache.keys() {
            println!("{k}");
        }
    }

    pub fn clean(&mut self) {
        self.cache.clear();
        self.persistent.set_len(0).unwrap();
        self.persistent.seek(SeekFrom::Start(0)).unwrap();
    }

    pub fn remove(&mut self, key: &str) {
        let row = Row::new_d(key.to_string());
        self.write_row(&row);
        if let Row::D { key } = row {
            self.cache.remove(&key);
        }
    }

    pub fn get(&self, key: &str) {
        match self.cache.get(key) {
            Some(v) => {
                println!("{}: {}", key, v);
            }
            None => {
                println!("value does not found with key: {}", key);
            }
        }
    }
}

impl Database {
    pub fn write_row(&mut self, row: &Row) {
        let row_line = format!("{}\n", row.ser());
        self.persistent
            .write(row_line.as_bytes())
            .expect("row write error");
        self.persistent.sync_all().expect("data sync to the file");
    }

    pub fn new() -> Self {
        let db_file = Self::db_file();
        Self::load(db_file)
    }

    fn load(file: File) -> Self {
        let reader = BufReader::new(&file);
        let mut cache = HashMap::new();
        for row in reader.lines() {
            let row = row.expect("something wrong while reading the database file");
            let row = Row::deser(&row).expect("Error in deser to the database row");
            match row {
                Row::I { key, value } => {
                    cache.insert(key, value);
                }
                Row::D { key } => {
                    cache.remove(&key);
                }
            }
        }
        Self {
            persistent: file,
            cache,
        }
    }

    fn db_file() -> File {
        let filename = "simple.db";
        match std::fs::File::options()
            .create(true)
            .read(true)
            .write(true)
            .append(true)
            .open(filename)
        {
            Ok(file) => file,
            Err(err) => {
                panic!("error in reading the database store file: {}", err)
            }
        }
    }
}

pub enum Row {
    I { key: String, value: String },
    D { key: String },
}

impl Row {
    pub fn new_i(key: String, value: String) -> Row {
        Self::I { key, value }
    }

    pub fn new_d(key: String) -> Row {
        Self::D { key }
    }

    const SEPARATOR: char = '|';
    pub fn deser(row: &str) -> Result<Self, String> {
        let parts = row.split(Self::SEPARATOR).collect::<Vec<_>>();
        if 0 == parts.len() {
            return Err("unsable to parse the row since it is empty".to_string());
        }

        let row = match parts[0] {
            "I" => {
                if parts.len() != 3 {
                    return Err(format!("unexpected-row-entry-length with I: {}", row));
                }

                Self::I {
                    key: parts[1].to_string(),
                    value: parts[2].to_string(),
                }
            }
            "D" => {
                if parts.len() != 2 {
                    return Err(format!("unexpected-row-entry-length with D: {}", row));
                }

                Self::D {
                    key: parts[1].to_string(),
                }
            }
            _ => return Err(format!("unexpected-row-entry: {}", row)),
        };

        Ok(row)
    }

    pub fn ser(&self) -> String {
        match self {
            Self::D { key } => {
                format!("D{}{key}", Self::SEPARATOR)
            }
            Self::I { key, value } => {
                format!("I{}{key}{}{value}", Self::SEPARATOR, Self::SEPARATOR)
            }
        }
    }
}
