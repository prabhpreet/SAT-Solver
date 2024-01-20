//DIMACS CNF parser
//https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/satformat.ps
use std::fs::File;
use std::io::{self, BufRead};

pub struct DimacsCnf {
    num_vars: usize,
    num_clauses: usize,
    clauses: Vec<Vec<i32>>,
}

pub struct DimacsCnfBuilder(DimacsCnf);

impl DimacsCnfBuilder {
    pub fn new() -> DimacsCnfBuilder {
        DimacsCnfBuilder(DimacsCnf::new())
    }

    fn parse_header(&mut self, header: &str) {
        let parts: Vec<&str> = header.split_whitespace().collect();
        if parts.len() >= 4 && parts[0] == "p" && parts[1] == "cnf" {
            self.0.num_vars = parts[2].parse().unwrap();
            self.0.num_clauses = parts[3].parse().unwrap();
        } else {
            panic!("Invalid DIMACS CNF header");
        }
    }

    fn parse_clause(&mut self, clause_line: &str) {
        let clause: Vec<i32> = clause_line
            .split_whitespace()
            .flat_map(|s| s.parse::<i32>())
            .collect();
        if !clause.is_empty() {
            self.0.clauses.push(clause);
        }
    }

    pub fn build(self) -> DimacsCnf {
        assert_eq!(self.0.clauses.len(), self.0.num_clauses);
        self.0
    }
}

impl DimacsCnf {
    fn new() -> DimacsCnf {
        DimacsCnf {
            num_vars: 0,
            num_clauses: 0,
            clauses: Vec::new(),
        }
    }

    pub fn clauses(&self) -> impl Iterator<Item = &Vec<i32>> {
        self.clauses.iter()
    }

    pub fn num_vars(&self) -> usize {
        self.num_vars
    }

    pub fn num_clauses(&self) -> usize {
        self.num_clauses
    }
}

pub fn parse_dimacs_cnf(file_path: &str) -> DimacsCnf {
    let mut dimacs_cnf = DimacsCnfBuilder::new();

    if let Ok(file) = File::open(file_path) {
        let reader = io::BufReader::new(file);
        let mut current_clause_line = String::new();

        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();

            if line.is_empty() || line.starts_with("c") {
                // Skip comments and empty lines
                continue;
            } else if line.starts_with("p cnf") {
                // Parse header
                dimacs_cnf.parse_header(line);
            } else {
                // Concatenate lines and split at each occurrence of 0
                let parts = line.split_whitespace();
                let mut is_first = true;

                for part in parts {
                    if part == "0" {
                        // If 0 is encountered, parse the clause and reset the current_clause_line
                        dimacs_cnf.parse_clause(&current_clause_line);
                        current_clause_line.clear();
                        is_first = true;
                    } else if is_first {
                        // If it's the first part, it might start with a negative sign
                        current_clause_line.push_str(part);
                        is_first = false;
                    } else {
                        // Otherwise, insert a space before adding the part to the current_clause_line
                        current_clause_line.push(' ');
                        current_clause_line.push_str(part);
                    }
                }
            }
        }

        // Check if there's a remaining clause in current_clause_line
        if !current_clause_line.is_empty() {
            dimacs_cnf.parse_clause(&current_clause_line);
        }
    } else {
        panic!("Failed to open file: {}", file_path);
    }

    dimacs_cnf.build()
}
