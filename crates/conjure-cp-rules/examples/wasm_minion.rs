//! Native and Emscripten smoke check for registration, parsing, rewriting and solving.
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use conjure_cp::ast::Name;
use conjure_cp::defaults::DEFAULT_RULE_SETS;
use conjure_cp::instantiate::validate_instantiation_conditions;
use conjure_cp::parse::tree_sitter::parse_essence;
use conjure_cp::representation::{get_repr_rules, util::try_up};
use conjure_cp::rule_engine::{get_all_rule_sets, get_all_rules, resolve_rule_sets, rewrite_model};
use conjure_cp::settings::{RewriteConfig, set_current_solver_family};
use conjure_cp::solver::{
    Solver,
    adaptors::{Minion, Sat},
};
use conjure_cp_rules as _;

fn main() {
    let rules = get_all_rules();
    let sets = get_all_rule_sets();
    let reprs = get_repr_rules().count();
    assert!(rules.len() > 250 && sets.len() > 10 && reprs > 25);
    println!(
        "{} rules, {} rule sets, {reprs} representations",
        rules.len(),
        sets.len()
    );

    assert!(parse_essence("find x :").is_err());
    check_solver(Solver::new(Minion::new()));
    check_solver(Solver::new(
        Sat::default().with_timeout(Some(std::time::Duration::from_secs(60))),
    ));
}

fn check_solver(solver: Solver) {
    let family = solver.get_family();
    set_current_solver_family(family);
    let (mut model, _) =
        parse_essence("find x, y, z : int(1..3)\nsuch that x != y, x != z, y != z\n").unwrap();
    validate_instantiation_conditions(&mut model).unwrap();
    let sets = resolve_rule_sets(family, DEFAULT_RULE_SETS).unwrap();
    let model = rewrite_model(&model, &sets, RewriteConfig::optimised()).unwrap();
    let declarations = ["x", "y", "z"].map(|name| {
        model
            .symbols_ptr_unchecked()
            .read()
            .lookup(&Name::User(name.into()))
            .unwrap()
    });
    let answers = Arc::new(Mutex::new(BTreeSet::new()));
    let collected = Arc::clone(&answers);
    solver
        .load_model(model)
        .unwrap()
        .solve(Box::new(move |solution| {
            let values = declarations
                .clone()
                .map(|decl| try_up(decl, &solution).unwrap().to_string());
            let mut sorted = values.clone();
            sorted.sort();
            assert_eq!(sorted, ["1", "2", "3"]);
            assert!(collected.lock().unwrap().insert(values));
            true
        }))
        .unwrap();
    assert_eq!(answers.lock().unwrap().len(), 6);
    println!("PASS: all six {family:?} solutions");
}
