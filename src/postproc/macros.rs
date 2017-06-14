// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[cfg(test)]
macro_rules! write_opt_for_tests {
    () => ({
        use WriteOptions;
        let mut opt = WriteOptions::default();
        opt.use_single_quote = true;
        opt.simplify_transform_matrices = true;
        opt
    })
}

#[cfg(test)]
macro_rules! base_test {
    ($name:ident, $functor:expr, $in_text:expr, $out_text:expr) => (
        #[test]
        fn $name() {
            let doc = Document::from_str($in_text).unwrap();
            $functor(&doc);
            assert_eq_text!(doc.to_string_with_opt(&write_opt_for_tests!()), $out_text);
        }
    )
}
