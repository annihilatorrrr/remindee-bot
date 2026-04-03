use crate::callbacks;
use crate::lang::Language;
use teloxide::types::{
    InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup,
};

pub(crate) struct ReminderMarkupEntry {
    pub(crate) text: String,
    pub(crate) rem_type: &'static str,
    pub(crate) rem_id: i64,
}

pub(crate) fn done_markup(lang: Language, occ_id: i64) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![InlineKeyboardButton::new(
        t!("Done", locale = lang.code()),
        InlineKeyboardButtonKind::CallbackData(callbacks::done_occurrence(
            occ_id,
        )),
    )])
}

pub(crate) fn edit_mode_markup(
    lang: Language,
    rem_id: i64,
) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![
        InlineKeyboardButton::new(
            t!("TimePattern", locale = lang.code()),
            InlineKeyboardButtonKind::CallbackData(
                callbacks::edit_mode_time_pattern(rem_id),
            ),
        ),
        InlineKeyboardButton::new(
            t!("Description", locale = lang.code()),
            InlineKeyboardButtonKind::CallbackData(
                callbacks::edit_mode_description(rem_id),
            ),
        ),
    ])
}

pub(crate) fn timezone_page_markup(
    prefixes: &[&'static str],
) -> InlineKeyboardMarkup {
    let mut markup = InlineKeyboardMarkup::default();

    for chunk in prefixes.chunks(2) {
        markup = markup.append_row(
            chunk
                .iter()
                .copied()
                .map(|prefix| {
                    InlineKeyboardButton::new(
                        prefix,
                        InlineKeyboardButtonKind::CallbackData(
                            callbacks::select_timezone_prefix(prefix),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    markup
}

pub(crate) fn timezone_child_page_markup(
    lang: Language,
    prefix: &str,
    num: usize,
    tz_names: Vec<(&'static str, &'static str)>,
    has_next_page: bool,
) -> InlineKeyboardMarkup {
    let mut markup = InlineKeyboardMarkup::default();

    for chunk in tz_names.chunks(2) {
        markup = markup.append_row(
            chunk
                .iter()
                .copied()
                .map(|(child_name, tz_name)| {
                    InlineKeyboardButton::new(
                        child_name,
                        InlineKeyboardButtonKind::CallbackData(
                            callbacks::select_timezone_tz(tz_name),
                        ),
                    )
                })
                .collect::<Vec<_>>(),
        );
    }

    let mut move_buttons = vec![];
    if num > 0 {
        move_buttons.push(InlineKeyboardButton::new(
            format!("‹ {}", t!("Prev", locale = lang.code())),
            InlineKeyboardButtonKind::CallbackData(
                callbacks::select_timezone_child_page(prefix, num - 1),
            ),
        ));
    }
    if has_next_page {
        move_buttons.push(InlineKeyboardButton::new(
            format!("{} ›", t!("Next", locale = lang.code())),
            InlineKeyboardButtonKind::CallbackData(
                callbacks::select_timezone_child_page(prefix, num + 1),
            ),
        ));
    }
    if !move_buttons.is_empty() {
        markup = markup.append_row(move_buttons);
    }

    markup.append_row(vec![InlineKeyboardButton::new(
        t!("ChooseAnotherRegion", locale = lang.code()),
        InlineKeyboardButtonKind::CallbackData(
            callbacks::select_timezone_back(),
        ),
    )])
}

pub(crate) fn languages_markup(
    languages: &[crate::lang::Language],
) -> InlineKeyboardMarkup {
    let row = languages
        .iter()
        .map(|lang| {
            InlineKeyboardButton::new(
                lang.name(),
                InlineKeyboardButtonKind::CallbackData(
                    callbacks::set_language(lang.code()),
                ),
            )
        })
        .collect::<Vec<_>>();
    InlineKeyboardMarkup::default().append_row(row)
}

pub(crate) fn settings_markup(lang: Language) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::default().append_row(vec![InlineKeyboardButton::new(
        t!("ChangeLanguage", locale = lang.code()),
        InlineKeyboardButtonKind::CallbackData(
            callbacks::settings_change_language(),
        ),
    )])
}

pub(crate) fn reminders_page_markup(
    num: usize,
    callback_kind: callbacks::ReminderListKind,
    reminders: Vec<ReminderMarkupEntry>,
    has_next_page: bool,
) -> InlineKeyboardMarkup {
    let mut markup = InlineKeyboardMarkup::default();

    for reminder in reminders {
        markup = markup.append_row(vec![InlineKeyboardButton::new(
            reminder.text,
            InlineKeyboardButtonKind::CallbackData(callbacks::reminder_alter(
                callback_kind,
                reminder.rem_type,
                reminder.rem_id,
            )),
        )]);
    }

    let mut move_buttons = vec![];
    if num > 0 {
        move_buttons.push(InlineKeyboardButton::new(
            "⬅️",
            InlineKeyboardButtonKind::CallbackData(callbacks::reminder_page(
                callback_kind,
                num - 1,
            )),
        ));
    }
    if has_next_page {
        move_buttons.push(InlineKeyboardButton::new(
            "➡️",
            InlineKeyboardButtonKind::CallbackData(callbacks::reminder_page(
                callback_kind,
                num + 1,
            )),
        ));
    }

    if move_buttons.is_empty() {
        markup
    } else {
        markup.append_row(move_buttons)
    }
}

#[cfg(test)]
mod tests {
    use super::timezone_child_page_markup;
    use crate::lang::Language;
    use teloxide::types::{
        InlineKeyboardButton, InlineKeyboardButtonKind::CallbackData,
        InlineKeyboardMarkup,
    };

    #[test]
    fn timezone_child_page_markup_uses_distinct_navigation_labels() {
        assert_eq!(
            timezone_child_page_markup(
                Language::English,
                "America",
                1,
                vec![("Chicago", "America/Chicago")],
                true,
            ),
            InlineKeyboardMarkup {
                inline_keyboard: vec![
                    vec![InlineKeyboardButton {
                        text: "Chicago".to_string(),
                        kind: CallbackData(
                            "seltz::tz::America/Chicago".to_string()
                        ),
                    }],
                    vec![
                        InlineKeyboardButton {
                            text: "‹ Prev".to_string(),
                            kind: CallbackData(
                                "seltz::child_page::America::0".to_string(),
                            ),
                        },
                        InlineKeyboardButton {
                            text: "Next ›".to_string(),
                            kind: CallbackData(
                                "seltz::child_page::America::2".to_string(),
                            ),
                        },
                    ],
                    vec![InlineKeyboardButton {
                        text: "Choose another region".to_string(),
                        kind: CallbackData("seltz::back".to_string()),
                    }],
                ],
            }
        );
    }
}
