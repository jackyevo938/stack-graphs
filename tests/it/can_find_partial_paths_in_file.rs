// -*- coding: utf-8 -*-
// ------------------------------------------------------------------------------------------------
// Copyright © 2021, stack-graphs authors.
// Licensed under either of Apache License, Version 2.0, or MIT license, at your option.
// Please see the LICENSE-APACHE or LICENSE-MIT files in this distribution for license details.
// ------------------------------------------------------------------------------------------------

use std::collections::HashSet;

use stack_graphs::graph::StackGraph;
use stack_graphs::partial::PartialPaths;

use crate::test_graphs;

fn check_partial_paths_in_file(graph: &StackGraph, file: &str, expected_paths: &[&str]) {
    let file = graph.get_file_unchecked(file);
    let mut partials = PartialPaths::new();
    let mut results = HashSet::new();
    partials.find_all_partial_paths_in_file(graph, file, |graph, partials, path| {
        if !path.is_complete_as_possible(graph) {
            return;
        }
        if !path.is_productive(partials) {
            return;
        }
        results.insert(path.display(graph, partials).to_string());
    });
    let expected_paths = expected_paths
        .iter()
        .map(|s| s.to_string())
        .collect::<HashSet<_>>();
    assert_eq!(results, expected_paths);
}

#[test]
fn class_field_through_function_parameter() {
    let fixture = test_graphs::class_field_through_function_parameter::new();
    check_partial_paths_in_file(
        &fixture.graph,
        "main.py",
        &[
            // definition of `__main__` module
            "<__main__> ($1) [root] -> [main.py(0) definition __main__] <> ($1)",
            // reference to `a` in import statement
            "<> () [main.py(17) reference a] -> [root] <a> ()",
            // `from a import *` means we can rewrite any lookup of `__main__.*` → `a.*`
            "<__main__.> ($1) [root] -> [root] <a.> ($1)",
            // reference to `b` in import statement
            "<> () [main.py(15) reference b] -> [root] <b> ()",
            // `from b import *` means we can rewrite any lookup of `__main__.*` → `b.*`
            "<__main__.> ($1) [root] -> [root] <b.> ($1)",
            // we can look for every reference in either `a` or `b`
            "<> () [main.py(9) reference A] -> [root] <a.A> ()",
            "<> () [main.py(9) reference A] -> [root] <b.A> ()",
            "<> () [main.py(10) reference bar] -> [root] <a.foo()/[main.py(7)].bar> ()",
            "<> () [main.py(10) reference bar] -> [root] <b.foo()/[main.py(7)].bar> ()",
            "<> () [main.py(13) reference foo] -> [root] <a.foo> ()",
            "<> () [main.py(13) reference foo] -> [root] <b.foo> ()",
            // parameter 0 of function call is `A`, which we can look up in either `a` or `b`
            "<0> ($1) [main.py(7) exported scope] -> [root] <a.A> ($1)",
            "<0> ($1) [main.py(7) exported scope] -> [root] <b.A> ($1)",
        ],
    );
    check_partial_paths_in_file(
        &fixture.graph,
        "a.py",
        &[
            // definition of `a` module
            "<a> ($1) [root] -> [a.py(0) definition a] <> ($1)",
            // definition of `foo` function
            "<a.foo> ($1) [root] -> [a.py(5) definition foo] <> ($1)",
            // reference to `x` in function body can resolve to formal parameter
            "<> () [a.py(8) reference x] -> [a.py(14) definition x] <> ()",
            // result of function is `x`, which is passed in as a formal parameter...
            "<a.foo()/$2> ($1) [root] -> [a.py(14) definition x] <> ()",
            // ...which we can look up either the 0th actual positional parameter...
            "<a.foo()/$2> ($1) [root] -> [jump to scope] <0> ($2)",
            // ...or the actual named parameter `x`
            "<a.foo()/$2> ($1) [root] -> [jump to scope] <x> ($2)",
        ],
    );
    check_partial_paths_in_file(
        &fixture.graph,
        "b.py",
        &[
            // definition of `b` module
            "<b> ($1) [root] -> [b.py(0) definition b] <> ($1)",
            // definition of class `A`
            "<b.A> ($1) [root] -> [b.py(5) definition A] <> ($1)",
            // definition of class member `A.bar`
            "<b.A.bar> ($1) [root] -> [b.py(8) definition bar] <> ($1)",
            // `bar` can also be accessed as an instance member
            "<b.A()/$2.bar> ($1) [root] -> [b.py(8) definition bar] <> ($2)",
        ],
    );
}

#[test]
fn sequenced_import_star() {
    let fixture = test_graphs::sequenced_import_star::new();
    check_partial_paths_in_file(
        &fixture.graph,
        "main.py",
        &[
            // definition of `__main__` module
            "<__main__> ($1) [root] -> [main.py(0) definition __main__] <> ($1)",
            // reference to `a` in import statement
            "<> () [main.py(8) reference a] -> [root] <a> ()",
            // `from a import *` means we can rewrite any lookup of `__main__.*` → `a.*`
            "<__main__.> ($1) [root] -> [root] <a.> ($1)",
            // reference to `foo` becomes `a.foo` because of import statement
            "<> () [main.py(6) reference foo] -> [root] <a.foo> ()",
        ],
    );
    check_partial_paths_in_file(
        &fixture.graph,
        "a.py",
        &[
            // definition of `a` module
            "<a> ($1) [root] -> [a.py(0) definition a] <> ($1)",
            // reference to `b` in import statement
            "<> () [a.py(6) reference b] -> [root] <b> ()",
            // `from b import *` means we can rewrite any lookup of `a.*` → `b.*`
            "<a.> ($1) [root] -> [root] <b.> ($1)",
        ],
    );
    check_partial_paths_in_file(
        &fixture.graph,
        "b.py",
        &[
            // definition of `b` module
            "<b> ($1) [root] -> [b.py(0) definition b] <> ($1)",
            // definition of `foo` inside of `b` module
            "<b.foo> ($1) [root] -> [b.py(5) definition foo] <> ($1)",
        ],
    );
}