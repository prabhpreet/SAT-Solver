
use sat_solver::{dimacs::parse_dimacs_cnf, definitions::CNF, dpll::{DPLLSolverBuilder}, Solver, SolverBuilder, pdpll::{PDPLLSolverBuilder}};
fn main() {
    pretty_env_logger::init();


    let files_path = "benchmarks/";

    //Get files in folder
    let files = std::fs::read_dir(files_path).unwrap();

    //Iterate over files
    let cnfs  = files.map(|file| {
        let file_path = file.unwrap().path();
        let file_path_str = file_path.to_str().unwrap();
        CNF::from(parse_dimacs_cnf(file_path_str))
    });


    for cnf in cnfs {
        let mut solvers : Vec<Box<dyn Solver>> = vec![];
        solvers.push(DPLLSolverBuilder::new().build(cnf.clone()));
        for i in 1..10 {
            solvers.push(PDPLLSolverBuilder::new(i).build(cnf.clone()));
        }

        for mut solver in solvers {
            println!("Satisfiability: {:?}", solver.solve());
        }
    }

}



