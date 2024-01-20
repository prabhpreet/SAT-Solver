use definitions::Satisfiability;

/*

Source:
    https://www.cse.iitb.ac.in/~akg/courses/2022-ar/lec-06-cdcl.pdf

Definitions:
    Literals: true, false or unassigned
    Clause C:
        C is true if l in C st l is true
        C is false if for each l in C, l is false
        Otherwise C is unassigned

    CNF F is true if for each C in F, C is true
    CNF F is false if there is C in F st C is false
    Otherwise CNF F is unassigned

    C is an unit clause under m if a literal l in C is unassigned and the rest are false
        l is a unit literal
*/
pub mod definitions;

pub trait SolverBuilder {
    fn build(self, f: crate::definitions::CNF) -> Box<dyn Solver>;
}

pub trait Solver {
    fn solve(&mut self) -> Satisfiability;
}

pub mod sdpll;
pub mod pdpll;
pub mod tests;
pub mod dimacs;