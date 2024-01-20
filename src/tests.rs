#[macro_export]
macro_rules! sat_tests {
    ($builder: expr) => {
    use crate::definitions::{ClauseBuilder, Literal, Satisfiability, CNF};
    use crate::{SolverBuilder};

    #[test]
    fn case_1() {
        let a = Literal::new("a".to_string());
        let b = Literal::new("b".to_string());
        let c = Literal::new("c".to_string());

        // CNF: (a v b) ^ (~a v c)
        let formula = CNF::new()
            .add_clause(
                ClauseBuilder::new()
                    .add_literal(a.positive())
                    .add_literal(b.positive()).build(),
            )
            .add_clause(
                ClauseBuilder::new()
                    .add_literal(a.complement())
                    .add_literal(b.positive()).build(),
            );

        assert_eq!($builder.build(formula).solve(), Satisfiability::SAT);
    }

    #[test]
    fn case_2() {
        //CNF: (a v b) ^ (~a v c) ^ (~b v ~c)
        let a = Literal::new("a".to_string());
        let b = Literal::new("b".to_string());
        let c = Literal::new("c".to_string());

        let formula = CNF::new()
            .add_clause(
                ClauseBuilder::new()
                    .add_literal(a.positive())
                    .add_literal(b.positive()).build(),
            )
            .add_clause(
                ClauseBuilder::new()
                    .add_literal(a.complement())
                    .add_literal(c.positive()).build(),
            )
            .add_clause(
                ClauseBuilder::new()
                    .add_literal(b.complement())
                    .add_literal(c.complement()).build(),
            );

        assert_eq!($builder.build(formula).solve(), Satisfiability::SAT);
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

        let c1 = ClauseBuilder::new()
            .add_literal(p1.complement())
            .add_literal(p2.positive()).build();
        let c2 = ClauseBuilder::new()
            .add_literal(p1.complement())
            .add_literal(p3.positive())
            .add_literal(p5.positive()).build();
        let c3 = ClauseBuilder::new()
            .add_literal(p2.complement())
            .add_literal(p4.positive()).build();
        let c4 = ClauseBuilder::new()
            .add_literal(p3.complement())
            .add_literal(p4.complement()).build();
        let c5 = ClauseBuilder::new()
            .add_literal(p1.positive())
            .add_literal(p5.positive())
            .add_literal(p2.complement()).build();
        let c6 = ClauseBuilder::new()
            .add_literal(p2.positive())
            .add_literal(p3.positive()).build();
        let c7 = ClauseBuilder::new()
            .add_literal(p2.positive())
            .add_literal(p3.complement())
            .add_literal(p7.positive()).build();
        let c8 = ClauseBuilder::new()
            .add_literal(p6.positive())
            .add_literal(p5.complement()).build();

        let formula = CNF::new()
            .add_clause(c1)
            .add_clause(c2)
            .add_clause(c3)
            .add_clause(c4)
            .add_clause(c5)
            .add_clause(c6)
            .add_clause(c7)
            .add_clause(c8);

        assert_eq!($builder.build(formula).solve(), Satisfiability::SAT);
    }

    #[test]
    fn case_4() {
        /*
           (x∨y∨z)∧(x∨y∨¬z)∧(x∨¬y∨z)∧(x∨¬y∨¬z)∧(¬x∨y∨z)∧(¬x∨y∨¬z)∧(¬x∨¬y∨z)∧(¬x∨¬y∨¬z)
        */

        let x = Literal::new("x".to_string());
        let y = Literal::new("y".to_string());
        let z = Literal::new("z".to_string());

        let c1 = ClauseBuilder::new()
            .add_literal(x.positive())
            .add_literal(y.positive())
            .add_literal(z.positive()).build();

        let c2 = ClauseBuilder::new()
            .add_literal(x.positive())
            .add_literal(y.positive())
            .add_literal(z.complement()).build();

        let c3 = ClauseBuilder::new()
            .add_literal(x.positive())
            .add_literal(y.complement())
            .add_literal(z.positive()).build();

        let c4 = ClauseBuilder::new()
            .add_literal(x.positive())
            .add_literal(y.complement())
            .add_literal(z.complement()).build();

        let c5 = ClauseBuilder::new()
            .add_literal(x.complement())
            .add_literal(y.positive())
            .add_literal(z.positive()).build();

        let c6 = ClauseBuilder::new()
            .add_literal(x.complement())
            .add_literal(y.positive())
            .add_literal(z.complement()).build();

        let c7 = ClauseBuilder::new()
            .add_literal(x.complement())
            .add_literal(y.complement())
            .add_literal(z.positive()).build();

        let c8 = ClauseBuilder::new()
            .add_literal(x.complement())
            .add_literal(y.complement())
            .add_literal(z.complement()).build();

        let formula = CNF::new()
            .add_clause(c1)
            .add_clause(c2)
            .add_clause(c3)
            .add_clause(c4)
            .add_clause(c5)
            .add_clause(c6)
            .add_clause(c7)
            .add_clause(c8);

        assert_eq!($builder.build(formula).solve(), Satisfiability::UNSAT);
    }
    };
}

pub use sat_tests;