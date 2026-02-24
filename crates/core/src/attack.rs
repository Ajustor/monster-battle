use crate::types::ElementType;
use serde::{Deserialize, Serialize};

/// Une attaque que le monstre peut utiliser en combat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attack {
    /// Nom de l'attaque.
    pub name: String,
    /// Type élémentaire de l'attaque.
    pub element: ElementType,
    /// Puissance de base.
    pub power: u32,
    /// Précision (0–100).
    pub accuracy: u8,
    /// Utilise special_attack/special_defense au lieu de attack/defense.
    pub is_special: bool,
}

impl Attack {
    /// Retourne les 4 attaques d'un monstre en fonction de ses types.
    ///
    /// - 1 attaque physique universelle (Charge, power 40, accuracy 100)
    /// - 2 attaques de type primaire
    /// - 1 attaque de type secondaire (si présent), sinon une 3ᵉ du type primaire.
    pub fn attacks_for_type(primary: ElementType, secondary: Option<ElementType>) -> Vec<Attack> {
        let mut attacks = vec![Attack {
            name: "Charge".to_string(),
            element: ElementType::Normal,
            power: 40,
            accuracy: 100,
            is_special: false,
        }];

        let primary_moves = Self::type_attacks(primary);
        // Prendre les 2 premières attaques du type primaire
        attacks.extend(primary_moves.iter().take(2).cloned());

        // 4ᵉ attaque : secondaire ou 3ᵉ primaire
        if let Some(sec) = secondary {
            let sec_moves = Self::type_attacks(sec);
            if let Some(first) = sec_moves.into_iter().next() {
                attacks.push(first);
            }
        } else if let Some(strong) = primary_moves.into_iter().last() {
            attacks.push(strong);
        }

        attacks.truncate(4);
        attacks
    }

    /// Attaques spécifiques à un type élémentaire (3 attaques de puissance croissante).
    fn type_attacks(element: ElementType) -> Vec<Attack> {
        match element {
            ElementType::Normal => vec![],
            ElementType::Fire => vec![
                Attack {
                    name: "Flammèche".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Griffe Brûlante".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: false,
                },
                Attack {
                    name: "Lance-Flammes".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
            ElementType::Water => vec![
                Attack {
                    name: "Pistolet à O".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Aqua-Queue".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: false,
                },
                Attack {
                    name: "Hydro-Canon".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
            ElementType::Plant => vec![
                Attack {
                    name: "Feuillage".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Fouet Liane".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: false,
                },
                Attack {
                    name: "Tempête Florale".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
            ElementType::Electric => vec![
                Attack {
                    name: "Étincelle".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Éclair".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: false,
                },
                Attack {
                    name: "Tonnerre".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
            ElementType::Earth => vec![
                Attack {
                    name: "Jet de Sable".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: false,
                },
                Attack {
                    name: "Séisme".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: false,
                },
                Attack {
                    name: "Éboulement".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: false,
                },
            ],
            ElementType::Wind => vec![
                Attack {
                    name: "Rafale".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Lame d'Air".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: true,
                },
                Attack {
                    name: "Tornade".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
            ElementType::Shadow => vec![
                Attack {
                    name: "Ombre Portée".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Griffe Ombre".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: false,
                },
                Attack {
                    name: "Cauchemar".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
            ElementType::Light => vec![
                Attack {
                    name: "Éclat".into(),
                    element,
                    power: 50,
                    accuracy: 100,
                    is_special: true,
                },
                Attack {
                    name: "Rayon Sacré".into(),
                    element,
                    power: 70,
                    accuracy: 90,
                    is_special: true,
                },
                Attack {
                    name: "Jugement".into(),
                    element,
                    power: 90,
                    accuracy: 75,
                    is_special: true,
                },
            ],
        }
    }
}
