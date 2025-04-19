use std::collections::HashMap;

use super::PatchUiNewFixture;

pub fn render_new_fixture_patch_name(
    new_fixture: PatchUiNewFixture,
    name_format: &str,
) -> Option<String> {
    interpolator::format(
        name_format,
        &[
            ("id", interpolator::Formattable::display(&new_fixture.id)),
            (
                "universe",
                interpolator::Formattable::display(&new_fixture.universe),
            ),
            (
                "address",
                interpolator::Formattable::display(&new_fixture.start_address),
            ),
            (
                "type",
                interpolator::Formattable::display(&new_fixture.type_and_mode),
            ),
            (
                "mode",
                interpolator::Formattable::display(&new_fixture.type_and_mode),
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>(),
    )
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_name_format_one() {
        assert_eq!(
            render_new_fixture_patch_name(
                PatchUiNewFixture {
                    id: 1,
                    universe: 42,
                    start_address: 11,
                    type_and_mode: 0,
                },
                "{type} {id} ({universe}.{address})"
            ),
            Some("Generic Wash Light 1 (42.11)".to_owned())
        );
    }
}
