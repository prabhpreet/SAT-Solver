use criterion::{Criterion, criterion_group, criterion_main};
use sat_solver::{dimacs::parse_dimacs_cnf, definitions::CNF, dpll::DPLLSolverBuilder, Solver, SolverBuilder, pdpll::PDPLLSolverBuilder};

pub fn criterion_benchmark(c: &mut Criterion) {
    let files_path = "benchmarks/";

    //Get files in folder
    let files = std::fs::read_dir(files_path).unwrap();

    //Iterate over files
    let cnfs: Vec<(String,CNF)>  = files.map(|file| {
        let file_path = file.unwrap().path();
        let file_path_str = file_path.to_str().unwrap();
        (file_path_str.to_string(),CNF::from(parse_dimacs_cnf(file_path_str)))
    }).collect();


    for (file,cnf) in cnfs {
        let mut group = c.benchmark_group(file);
        let mut solvers : Vec<(String,Box<dyn Solver>)> = vec![];

        solvers.push(("Single Threaded".to_string(),DPLLSolverBuilder::new().build(cnf.clone())));
        for i in 1..5 {
            solvers.push(("Parallel factor:".to_string() + &i.to_string(),PDPLLSolverBuilder::new(i).build(cnf.clone())));
        }

        for (id, mut solver) in solvers {
            group.bench_function(&id, |b| b.iter(|| solver.solve()));
        }
        group.finish()
    }

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);


