use crate::{
    definitions::{Assignments, CNFValue, LiteralValue, Satisfiability, CNF, SignedLiteral, RefLiteral},
    Solver, SolverBuilder,
};
use rand::{Rng, seq::IteratorRandom, rngs, thread_rng};
use log::debug;

pub struct DPLLSolverBuilder {
}

impl DPLLSolverBuilder {
    pub fn new() -> Self {
        DPLLSolverBuilder { }
    }

}

impl SolverBuilder for DPLLSolverBuilder {
    fn build(self, formula: CNF) -> Box<dyn Solver> {
        Box::new(DPLLSolver{formula})
    }
}

pub struct DPLLSolver {
    formula: CNF,
}

impl Solver for DPLLSolver {
    fn solve(&mut self) -> Satisfiability {
        let func = |f: CNF| f.mom(1);
        let di = if true {
            DI::LazyLiteralList(func)
        } else {
            DI::RandomDecision
        };

        //Pure literal elimination- if a literal l only appears as positive/negative, assign it to true/false
        //to strive for satisfiability
        let mut m = Assignments::new();
        self.formula.pure_literals().iter().for_each(|l| {
            let value = match l {
                SignedLiteral::Id(_) => LiteralValue::True,
                SignedLiteral::Not(_) => LiteralValue::False,
            };
            debug!("Pure literal elimination: Literal {:?}, {:?}", l, value);
            m.assign(l.literal(), value);
        });

        self.dpll_recursive(CNFValue::Formula(self.formula.clone()), m, di)
            .unwrap()
    }
}

#[derive(Debug,PartialEq)]
//Next iteration decision instruction
enum DI<F: Fn(CNF)->Vec<RefLiteral>> {
    RandomDecision,
    LiteralList(Vec<RefLiteral>),
    LazyLiteralList(F),
}

impl DPLLSolver {
    /*
        DPLL(F,m):
        Input: CNF, partial assigment m
        Output: SAT/UNSAT
    */
    fn dpll_recursive<F: Fn(CNF)->Vec<RefLiteral>>(&self, formula: CNFValue, m: Assignments, di: DI<F>) -> CNFValue {
        match formula.evaluate(&m) {
            CNFValue::Formula(f) => {
                //Unit propogation- unit p becomes a unit literal for some clause
                let m = || -> Option<Assignments> {
                    for clause in f.clauses() {
                        if let Some(l) = clause.is_unit_clause() {
                            let value = match l {
                                SignedLiteral::Id(_) => LiteralValue::True,
                                SignedLiteral::Not(_) => LiteralValue::False,
                            };
                            debug!("Unit propogation: Clause {:?}, {:?}", clause, value);
                            return Some(Assignments::new().assign(l.literal(), value).to_owned());
                        }
                    }
                    None
                }();

                if let Some(m) = m {
                    return self.dpll_recursive::<F>(CNFValue::Formula(f.clone()), m, DI::RandomDecision);
                }

                //Decision: If at this point in time we don't find a unit literal or reach a base case, let's make a decision

                //First decision: Choose an unassigned literal p and a random bit b in {0,1} and check for satisfiability
                let p = match di {
                    DI::RandomDecision => f.iter_literals()
                        .choose(&mut thread_rng()).unwrap().literal(),
                    DI::LiteralList(ref l) => {
                        if let Some(l) = l.first() {
                            l.to_owned()
                        } else {
                            f.iter_literals()
                                .choose(&mut thread_rng()).unwrap().literal()
                        }
                    },
                    DI::LazyLiteralList(ref func) => {
                        if let Some(l) = func(f.clone()).first() {
                            l.to_owned()
                        } else {
                            f.iter_literals()
                                .choose(&mut thread_rng()).unwrap().literal()
                        }
                    },
                };
                
                //Positive bias
                let value = LiteralValue::True;

                debug!("Set {:?} to {:?}", p, value);
                if self.dpll_recursive::<F>(
                    CNFValue::Formula(f.clone()),
                    Assignments::new().assign(p.clone(), value).to_owned(), DI::RandomDecision
                ) == CNFValue::SAT
                {
                    debug!("SAT: {:?}", m);
                    return CNFValue::SAT;
                }
                // Let's backtrack in case the first decision doesn't work out
                else {
                    return self.dpll_recursive::<F>(
                        CNFValue::Formula(f.clone()),
                        Assignments::new()
                            .assign(p.clone(), value.negate())
                            .to_owned(), DI::RandomDecision
                    );
                }
            }

            //Base cases: if F is satisfied/unsatisfied by assignments
            v => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    crate::tests::sat_tests!(DPLLSolverBuilder::new());
}