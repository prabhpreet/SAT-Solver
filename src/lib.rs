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

pub trait Solver {
    fn new(f: crate::definitions::CNF) -> Self;
    fn solve(self) -> Satisfiability;
}

/*
DPLL
    Input: CNF F
    Output: SAT/UNSAT
    Return: DPLL(F,phi) : Recursive

    - Maintains a partial model, initially phi
    - Assigns unassigned variables 0 or 1 randomly one after another
    - Sometimes forced to chose assignments due to unit literals

DPLL(F,m):
    Input: CNF, partial assigment m
    Output: SAT/UNSAT
*/
pub mod dpll;

#[cfg(test)]
mod test {
    use crate::definitions::{Clause, Literal, Satisfiability, CNF};
    use crate::dpll::DPLLSolver;
    use crate::Solver;
    #[test]
    fn case_1() {
        let a = Literal::new("a".to_string());
        let b = Literal::new("b".to_string());
        let c = Literal::new("c".to_string());

        // CNF: (a v b) ^ (~a v c)
        let formula = CNF::new()
            .add_clause(
                Clause::new()
                    .add_literal(a.positive())
                    .add_literal(b.positive()),
            )
            .add_clause(
                Clause::new()
                    .add_literal(a.negated())
                    .add_literal(c.positive()),
            );

        assert_eq!(DPLLSolver::new(formula).solve(), Satisfiability::SAT);
    }

    #[test]
    fn case_2() {
        //CNF: (a v b) ^ (~a v c) ^ (~b v ~c)
        let a = Literal::new("a".to_string());
        let b = Literal::new("b".to_string());
        let c = Literal::new("c".to_string());

        let formula = CNF::new()
            .add_clause(
                Clause::new()
                    .add_literal(a.positive())
                    .add_literal(b.positive()),
            )
            .add_clause(
                Clause::new()
                    .add_literal(a.negated())
                    .add_literal(c.positive()),
            )
            .add_clause(
                Clause::new()
                    .add_literal(b.negated())
                    .add_literal(c.negated()),
            );

        assert_eq!(DPLLSolver::new(formula).solve(), Satisfiability::SAT);
    }

    #[test]
    fn case_3() {
        /*
           c1 = (¬p1 ∨ p2)
           c2 = (¬p1 ∨ p3 ∨ p5)
           c3 = (¬p2 ∨ p4)
           c4 = (¬p3 ∨ ¬p4)
           c5 = (p1 ∨ p5 ∨ ¬p2)
           c6 = (p2 ∨ p3)
           c7 = (p2 ∨ ¬p3 ∨ p7)
           c8 = (p6 ∨ ¬p5)
        */

        let p1 = Literal::new("p1".to_string());
        let p2 = Literal::new("p2".to_string());
        let p3 = Literal::new("p3".to_string());
        let p4 = Literal::new("p4".to_string());
        let p5 = Literal::new("p5".to_string());
        let p6 = Literal::new("p6".to_string());
        let p7 = Literal::new("p7".to_string());

        let c1 = Clause::new()
            .add_literal(p1.negated())
            .add_literal(p2.positive());
        let c2 = Clause::new()
            .add_literal(p1.negated())
            .add_literal(p3.positive())
            .add_literal(p5.positive());
        let c3 = Clause::new()
            .add_literal(p2.negated())
            .add_literal(p4.positive());
        let c4 = Clause::new()
            .add_literal(p3.negated())
            .add_literal(p4.negated());
        let c5 = Clause::new()
            .add_literal(p1.positive())
            .add_literal(p5.positive())
            .add_literal(p2.negated());
        let c6 = Clause::new()
            .add_literal(p2.positive())
            .add_literal(p3.positive());
        let c7 = Clause::new()
            .add_literal(p2.positive())
            .add_literal(p3.negated())
            .add_literal(p7.positive());
        let c8 = Clause::new()
            .add_literal(p6.positive())
            .add_literal(p5.negated());

        let formula = CNF::new()
            .add_clause(c1)
            .add_clause(c2)
            .add_clause(c3)
            .add_clause(c4)
            .add_clause(c5)
            .add_clause(c6)
            .add_clause(c7)
            .add_clause(c8);

        assert_eq!(DPLLSolver::new(formula).solve(), Satisfiability::SAT);
    }

    #[test]
    fn case_4() {
        /*
           (x∨y∨z)∧(x∨y∨¬z)∧(x∨¬y∨z)∧(x∨¬y∨¬z)∧(¬x∨y∨z)∧(¬x∨y∨¬z)∧(¬x∨¬y∨z)∧(¬x∨¬y∨¬z)
        */

        let x = Literal::new("x".to_string());
        let y = Literal::new("y".to_string());
        let z = Literal::new("z".to_string());

        let c1 = Clause::new()
            .add_literal(x.positive())
            .add_literal(y.positive())
            .add_literal(z.positive());

        let c2 = Clause::new()
            .add_literal(x.positive())
            .add_literal(y.positive())
            .add_literal(z.negated());

        let c3 = Clause::new()
            .add_literal(x.positive())
            .add_literal(y.negated())
            .add_literal(z.positive());

        let c4 = Clause::new()
            .add_literal(x.positive())
            .add_literal(y.negated())
            .add_literal(z.negated());

        let c5 = Clause::new()
            .add_literal(x.negated())
            .add_literal(y.positive())
            .add_literal(z.positive());

        let c6 = Clause::new()
            .add_literal(x.negated())
            .add_literal(y.positive())
            .add_literal(z.negated());

        let c7 = Clause::new()
            .add_literal(x.negated())
            .add_literal(y.negated())
            .add_literal(z.positive());

        let c8 = Clause::new()
            .add_literal(x.negated())
            .add_literal(y.negated())
            .add_literal(z.negated());

        let formula = CNF::new()
            .add_clause(c1)
            .add_clause(c2)
            .add_clause(c3)
            .add_clause(c4)
            .add_clause(c5)
            .add_clause(c6)
            .add_clause(c7)
            .add_clause(c8);

        assert_eq!(DPLLSolver::new(formula).solve(), Satisfiability::UNSAT);
    }
}
