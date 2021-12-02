use std::io;

use system::*;

pub struct TSV {
    pub header: Vec<String>,
    pub data: Vec<Vec<String>>
}
impl TSV {
    pub fn new(header: Vec<&str>) -> TSV {
        TSV{
            header: header.iter().map(|s| { s.to_string() }).collect(),
            data: vec!()
        }
    }

    pub fn add_row(&mut self, row: Vec<String>) {
        let len = self.header.len();
        if row.len() != self.header.len() {
            panic!("TSV: Wrong row size. Expected {}, found {}.",
                   len, row.len());
        }
        self.data.push(row);        
    }
    
    pub fn arrange(&mut self, new_header: Vec<&str>) {
        let mut column_order = vec!(); // column_order[name_index] = actual_index
        if self.header.len() != new_header.len() {
            panic!("TSV arrange: expected {} columns but found {}",
                   new_header.len(), self.header.len());
        }
        for name in new_header.iter() {
            let mut found = false;
            for (i, col) in self.header.iter().enumerate() {
                if col == name {
                    column_order.push(i);
                    found = true;
                }
            }
            if !found {
                panic!("TSV: column {} not found", name);
            }
        }
        if column_order.len() != new_header.len() {
            panic!("TSV: invalid column names; expected {:?}", new_header);
        }
        let mut new_rows = vec!();
        for row in self.data.iter() {
            let mut new_row = vec!();
            for &i in column_order.iter() {
                new_row.push(row[i].clone()); // TODO: needless copying
            }
            new_rows.push(new_row);
        }
        self.header = new_header.iter().map(|s| { s.to_string() }).collect();
        self.data = new_rows;
    }
}

pub fn parse_number(s: &str) -> isize {
    s.parse::<isize>().expect(&format!("Invalid number"))
}

pub fn parse_name(name: &str) -> char {
    if name.len() != 1 {
        panic!("Invalid name: {}", name)
    }
    name.chars().next().unwrap() // first char
}

pub fn parse_u8(s: &str) -> u8 {
    u8::from_str_radix(s, 16)
        .expect(&format!("Invalid color component: {}", s))
}

pub fn parse_color(s: &str) -> (u8, u8, u8) {
    if s.len() != 7 || !s.starts_with("#") {
        panic!("Invalid s: {}", s)
    }
    let r = parse_u8(&s[1..3]);
    let g = parse_u8(&s[3..5]);
    let b = parse_u8(&s[5..7]);
    (r, g, b)
}

impl Loadable for TSV {
    fn load(file: &mut RFile) -> TSV {
        let mut rows = vec!();
        let filename = file.name().to_string();
        let mut lines = file.lines();

        /* Read headers */
        let header = lines.next()
            .expect(&format!("TSV {}: file is empty", filename))
            .unwrap();
        let columns: Vec<&str> = header.split("\t").collect();

        /* Read rows */
        for line in lines {
            let line = match line {
                Err(err) => panic!("TSV {}: io error. {}", filename, err),
                Ok(line) => line
            };
            if line.starts_with("//") { continue; }
            let row: Vec<String> = line.split("\t").map(|s| { s.to_string() }).collect();
            if row.len() != columns.len() {
                panic!("TSV {}: wrong row size. Expected {}, found {}",
                       filename, columns.len(), row.len());
            }
            rows.push(row);
        }

        TSV{
            header: columns.iter().map(|s| { s.to_string() }).collect(),
            data: rows
        }
    }
}

impl Savable for TSV {
    fn save(self, file: &mut WFile) {
        let ans = || -> io::Result<()> {
            let len = self.header.len();
            for (i, name) in self.header.iter().enumerate() {
                try!(write!(file, "{}", name));
                if i != len - 1 {
                    try!(write!(file, "\t"));
                }
            }
            try!(write!(file, "\n"));
            for row in self.data.iter() {
                for (i, cell) in row.iter().enumerate() {
                    try!(write!(file, "{}", cell));
                    if i != len - 1 {
                        try!(write!(file, "\t"));
                    }
                }
                try!(write!(file, "\n"));
            }
            Ok(())
        }();
        match ans {
            Err(io_err) => panic!("TSV {}: Failed to write to file. {}", file.name(), io_err),
            Ok(()) => ()
        }
    }
}
