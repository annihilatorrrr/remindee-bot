#[cfg(not(test))]
use crate::db::Database;
#[cfg(test)]
use crate::db::MockDatabase as Database;
use crate::err;
use std::sync::LazyLock;

use chrono_tz::Tz;
use teloxide::types::UserId;
use tzf_rs::DefaultFinder;

const TZ_PAGE_SIZE: usize = 30;

type TzGroup = &'static [(&'static str, &'static str)];

macro_rules! define_tz_data {
    ($( $prefix:literal => [$($name:literal),* $(,)?] ),* $(,)?) => {
        pub(crate) fn get_tz_prefixes() -> &'static [&'static str] {
            &[$($prefix),*]
        }

        fn get_tz_group(prefix: &str) -> Option<TzGroup> {
            match prefix {
                $(
                    $prefix => Some(&[
                        $(($name, concat!($prefix, "/", $name))),*
                    ]),
                )*
                _ => None,
            }
        }
    };
}

define_tz_data! {
    "Africa" => [
        "Abidjan",
        "Accra",
        "Algiers",
        "Bissau",
        "Cairo",
        "Casablanca",
        "Ceuta",
        "El_Aaiun",
        "Johannesburg",
        "Juba",
        "Khartoum",
        "Lagos",
        "Maputo",
        "Monrovia",
        "Nairobi",
        "Ndjamena",
        "Sao_Tome",
        "Tripoli",
        "Tunis",
        "Windhoek",
    ],
    "America" => [
        "Adak",
        "Anchorage",
        "Araguaina",
        "Argentina/Buenos_Aires",
        "Argentina/Catamarca",
        "Argentina/Cordoba",
        "Argentina/Jujuy",
        "Argentina/La_Rioja",
        "Argentina/Mendoza",
        "Argentina/Rio_Gallegos",
        "Argentina/Salta",
        "Argentina/San_Juan",
        "Argentina/San_Luis",
        "Argentina/Tucuman",
        "Argentina/Ushuaia",
        "Asuncion",
        "Atikokan",
        "Bahia",
        "Bahia_Banderas",
        "Barbados",
        "Belem",
        "Belize",
        "Blanc-Sablon",
        "Boa_Vista",
        "Bogota",
        "Boise",
        "Cambridge_Bay",
        "Campo_Grande",
        "Cancun",
        "Caracas",
        "Cayenne",
        "Chicago",
        "Chihuahua",
        "Ciudad_Juarez",
        "Costa_Rica",
        "Coyhaique",
        "Creston",
        "Cuiaba",
        "Curacao",
        "Danmarkshavn",
        "Dawson",
        "Dawson_Creek",
        "Denver",
        "Detroit",
        "Edmonton",
        "Eirunepe",
        "El_Salvador",
        "Fort_Nelson",
        "Fortaleza",
        "Glace_Bay",
        "Goose_Bay",
        "Grand_Turk",
        "Guatemala",
        "Guayaquil",
        "Guyana",
        "Halifax",
        "Havana",
        "Hermosillo",
        "Indiana/Indianapolis",
        "Indiana/Knox",
        "Indiana/Marengo",
        "Indiana/Petersburg",
        "Indiana/Tell_City",
        "Indiana/Vevay",
        "Indiana/Vincennes",
        "Indiana/Winamac",
        "Inuvik",
        "Iqaluit",
        "Jamaica",
        "Juneau",
        "Kentucky/Louisville",
        "Kentucky/Monticello",
        "La_Paz",
        "Lima",
        "Los_Angeles",
        "Maceio",
        "Managua",
        "Manaus",
        "Martinique",
        "Matamoros",
        "Mazatlan",
        "Menominee",
        "Merida",
        "Metlakatla",
        "Mexico_City",
        "Miquelon",
        "Moncton",
        "Monterrey",
        "Montevideo",
        "Nassau",
        "New_York",
        "Nipigon",
        "Nome",
        "Noronha",
        "North_Dakota/Beulah",
        "North_Dakota/Center",
        "North_Dakota/New_Salem",
        "Nuuk",
        "Ojinaga",
        "Panama",
        "Pangnirtung",
        "Paramaribo",
        "Phoenix",
        "Port-au-Prince",
        "Port_of_Spain",
        "Porto_Velho",
        "Puerto_Rico",
        "Punta_Arenas",
        "Rainy_River",
        "Rankin_Inlet",
        "Recife",
        "Regina",
        "Resolute",
        "Rio_Branco",
        "Santarem",
        "Santiago",
        "Santo_Domingo",
        "Sao_Paulo",
        "Scoresbysund",
        "Sitka",
        "St_Johns",
        "Swift_Current",
        "Tegucigalpa",
        "Thule",
        "Thunder_Bay",
        "Tijuana",
        "Toronto",
        "Vancouver",
        "Whitehorse",
        "Winnipeg",
        "Yakutat",
        "Yellowknife",
    ],
    "Antarctica" => [
        "Casey",
        "Davis",
        "DumontDUrville",
        "Macquarie",
        "Mawson",
        "Palmer",
        "Rothera",
        "Syowa",
        "Troll",
        "Vostok",
    ],
    "Asia" => [
        "Almaty",
        "Amman",
        "Anadyr",
        "Aqtau",
        "Aqtobe",
        "Ashgabat",
        "Atyrau",
        "Baghdad",
        "Baku",
        "Bangkok",
        "Barnaul",
        "Beirut",
        "Bishkek",
        "Brunei",
        "Chita",
        "Choibalsan",
        "Colombo",
        "Damascus",
        "Dhaka",
        "Dili",
        "Dubai",
        "Dushanbe",
        "Famagusta",
        "Gaza",
        "Hebron",
        "Ho_Chi_Minh",
        "Hong_Kong",
        "Hovd",
        "Irkutsk",
        "Jakarta",
        "Jayapura",
        "Jerusalem",
        "Kabul",
        "Kamchatka",
        "Karachi",
        "Kathmandu",
        "Khandyga",
        "Kolkata",
        "Krasnoyarsk",
        "Kuala_Lumpur",
        "Kuching",
        "Macau",
        "Magadan",
        "Makassar",
        "Manila",
        "Nicosia",
        "Novokuznetsk",
        "Novosibirsk",
        "Omsk",
        "Oral",
        "Pontianak",
        "Pyongyang",
        "Qatar",
        "Qostanay",
        "Qyzylorda",
        "Riyadh",
        "Sakhalin",
        "Samarkand",
        "Seoul",
        "Shanghai",
        "Singapore",
        "Srednekolymsk",
        "Taipei",
        "Tashkent",
        "Tbilisi",
        "Tehran",
        "Thimphu",
        "Tokyo",
        "Tomsk",
        "Ulaanbaatar",
        "Urumqi",
        "Ust-Nera",
        "Vladivostok",
        "Yakutsk",
        "Yangon",
        "Yekaterinburg",
        "Yerevan",
    ],
    "Atlantic" => [
        "Azores",
        "Bermuda",
        "Canary",
        "Cape_Verde",
        "Faroe",
        "Madeira",
        "Reykjavik",
        "South_Georgia",
        "Stanley",
    ],
    "Australia" => [
        "Adelaide",
        "Brisbane",
        "Broken_Hill",
        "Currie",
        "Darwin",
        "Eucla",
        "Hobart",
        "Lindeman",
        "Lord_Howe",
        "Melbourne",
        "Perth",
        "Sydney",
    ],
    "Europe" => [
        "Amsterdam",
        "Andorra",
        "Astrakhan",
        "Athens",
        "Belgrade",
        "Berlin",
        "Brussels",
        "Bucharest",
        "Budapest",
        "Chisinau",
        "Copenhagen",
        "Dublin",
        "Gibraltar",
        "Helsinki",
        "Istanbul",
        "Kaliningrad",
        "Kirov",
        "Kyiv",
        "Lisbon",
        "London",
        "Luxembourg",
        "Madrid",
        "Malta",
        "Minsk",
        "Monaco",
        "Moscow",
        "Oslo",
        "Paris",
        "Prague",
        "Riga",
        "Rome",
        "Samara",
        "Saratov",
        "Simferopol",
        "Sofia",
        "Stockholm",
        "Tallinn",
        "Tirane",
        "Ulyanovsk",
        "Uzhgorod",
        "Vienna",
        "Vilnius",
        "Volgograd",
        "Warsaw",
        "Zaporozhye",
        "Zurich",
    ],
    "Indian" => [
        "Chagos",
        "Christmas",
        "Cocos",
        "Kerguelen",
        "Mahe",
        "Maldives",
        "Mauritius",
        "Reunion",
    ],
    "Pacific" => [
        "Apia",
        "Auckland",
        "Bougainville",
        "Chatham",
        "Chuuk",
        "Easter",
        "Efate",
        "Fakaofo",
        "Fiji",
        "Funafuti",
        "Galapagos",
        "Gambier",
        "Guadalcanal",
        "Guam",
        "Honolulu",
        "Kanton",
        "Kiritimati",
        "Kosrae",
        "Kwajalein",
        "Majuro",
        "Marquesas",
        "Nauru",
        "Niue",
        "Norfolk",
        "Noumea",
        "Pago_Pago",
        "Palau",
        "Pitcairn",
        "Pohnpei",
        "Port_Moresby",
        "Rarotonga",
        "Tahiti",
        "Tarawa",
        "Tongatapu",
        "Wake",
        "Wallis",
    ],
}

pub(crate) fn get_tz_names_for_prefix_page(
    prefix: &str,
    num: usize,
) -> Option<Vec<(&'static str, &'static str)>> {
    get_tz_group(prefix)?
        .chunks(TZ_PAGE_SIZE)
        .nth(num)
        .map(|tz_names| tz_names.to_vec())
}

pub(crate) async fn get_user_timezone(
    db: &Database,
    user_id: UserId,
) -> Result<Option<Tz>, err::Error> {
    let tz_name_opt = db.get_user_timezone_name(user_id.0 as i64).await?;
    tz_name_opt
        .map(|tz_name| tz_name.parse::<Tz>().map_err(err::Error::Parse))
        .transpose()
}

pub(crate) fn get_timezone_name_of_location(
    lng: f64,
    lat: f64,
) -> &'static str {
    static FINDER: LazyLock<DefaultFinder> = LazyLock::new(DefaultFinder::new);
    FINDER.get_tz_name(lng, lat)
}

#[cfg(test)]
mod tests {
    use super::{get_tz_names_for_prefix_page, get_tz_prefixes};

    #[test]
    fn timezone_prefixes_keep_source_order() {
        assert_eq!(
            get_tz_prefixes(),
            vec![
                "Africa",
                "America",
                "Antarctica",
                "Asia",
                "Atlantic",
                "Australia",
                "Europe",
                "Indian",
                "Pacific",
            ]
        );
    }

    #[test]
    fn timezone_children_are_relative_to_prefix() {
        assert_eq!(
            get_tz_names_for_prefix_page("Antarctica", 0),
            Some(vec![
                ("Casey", "Antarctica/Casey"),
                ("Davis", "Antarctica/Davis"),
                ("DumontDUrville", "Antarctica/DumontDUrville"),
                ("Macquarie", "Antarctica/Macquarie"),
                ("Mawson", "Antarctica/Mawson"),
                ("Palmer", "Antarctica/Palmer"),
                ("Rothera", "Antarctica/Rothera"),
                ("Syowa", "Antarctica/Syowa"),
                ("Troll", "Antarctica/Troll"),
                ("Vostok", "Antarctica/Vostok"),
            ])
        );
    }

    #[test]
    fn timezone_children_keep_nested_regions_relative_to_prefix() {
        assert!(get_tz_names_for_prefix_page("America", 0)
            .unwrap()
            .contains(&(
                "Argentina/Buenos_Aires",
                "America/Argentina/Buenos_Aires",
            )));
    }

    #[test]
    fn timezone_children_reject_invalid_prefixes() {
        assert_eq!(get_tz_names_for_prefix_page("Mars", 0), None);
    }
}
