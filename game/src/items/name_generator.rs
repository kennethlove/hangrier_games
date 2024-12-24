use rand::prelude::*;

// Adjectives to come before "shield"
// _____ shield
const SHIELD_ADJECTIVES: &[&str] = &[
    "iron",
    "wooden",
    "brass",
    "bronze",
    "glass",
    "steel",
    "stone",
];

// Weapon nouns
// <descriptor> _____
const WEAPON_NOUNS: &[&str] = &[
    "sword",
    "spear",
    "dagger",
    "knife",
    "net",
    "trident",
    "bow",
    "mace",
];

// Adjectives to come before a weapon noun
// _____ <weapon noun>
const WEAPON_ADJECTIVES: &[&str] = &[
    "sharp",
    "heavy",
    "long",
    "short",
    "glass",
    "iron",
    "wooden",
    "brass",
    "bronze",
    "glass",
    "steel",
    "stone",
];



pub fn generate_shield_name() -> String {
    let mut rng = thread_rng();
    let adjective = SHIELD_ADJECTIVES.choose(&mut rng).unwrap().to_owned();
    format!("{} {}", adjective, "shield")
}

pub fn generate_weapon_name() -> String {
    let mut rng = thread_rng();
    let adjective = WEAPON_ADJECTIVES.choose(&mut rng).unwrap().to_owned();
    let noun = WEAPON_NOUNS.choose(&mut rng).unwrap().to_owned();
    format!("{} {}", adjective, noun)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shield_name() {
        let name = generate_shield_name();
        assert!(name.contains(" shield"));
    }

    #[test]
    fn weapon_name() {
        let name = generate_weapon_name();
        assert!(name.contains(" "));

        let mut name = name.as_str().split(" ");
        let adjective = name.next().unwrap();
        assert!(WEAPON_ADJECTIVES.contains(&adjective));
        let noun = name.next().unwrap();
        assert!(WEAPON_NOUNS.contains(&noun));
    }
}
