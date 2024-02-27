use xee_xpath::{sequence::Item, Session};

pub(crate) fn convert_string(_: &mut Session, item: &Item) -> xee_xpath::error::Result<String> {
    item.to_atomic()?.try_into()
}

pub(crate) fn convert_boolean(
    session: &mut Session,
    item: &Item,
) -> xee_xpath::error::Result<bool> {
    Ok(convert_string(session, item)? == "true")
}
