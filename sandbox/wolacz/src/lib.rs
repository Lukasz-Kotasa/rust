
struct Osoba {
    first_name: String,
}

trait Deklinacja {
    fn wolacz(self: &mut Self) -> String;
}

fn usun_przedostatnia_e(s: String) -> String {
    let mut result = s.clone();

    match result.chars().rev().enumerate().nth(1).unwrap().1 {
        'e' => result.remove(s.len() - 2),
        _ => ' ',
    };
    result
}

impl Deklinacja for Osoba {
    fn wolacz(self: &mut Self) -> String {
        let mut ret = self.first_name.clone();

        // jesli przedostatnia litera jest 'e', usuń ją (wyjątek: nie usuwaj, jeśli kończy się na 'm' 'j' 'w' 'l'
        // NikodEm -> NikodEmie, BartlomiEj -> BartlomiEju, MaciEj -> MaciEju, ZbigniEw -> ZbigniEwie, DaniEl -> DaniElu
        if !ret.ends_with("m") && !ret.ends_with("j") && !ret.ends_with("w") && !ret.ends_with("iel"){
            // MarEk -> Marku, AntEk -> Antku, BartEk -> Bartku, FranciszEk -> Franciszku, KacpEr -> Kacprze, etc
            ret = usun_przedostatnia_e(ret);
        }
            
        if ret.ends_with("l") && !ret.ends_with("iel") {
            // Michal, Pawel ale nie Daniel
            ret.push_str("e");
        } else if ret.ends_with("r") {
            // Wiktor, Waldemar, Kacper
            ret.push_str("ze");
        } else if ret.ends_with("n") || ret.ends_with("aw") || ret.ends_with("em") || ret.ends_with("am")
              || ret.ends_with("ub") || ret.ends_with("ip") || ret.ends_with("ew") {
            // Filip, Nikodem, Szymon, Stanislaw, Wieslaw, Jakub, Jan, Zbigniew
            ret.push_str("ie");
        } else if ret.ends_with("a") {
            // imiona żeńskie
            ret.truncate(ret.len() - 1);
            ret.push_str("o");
        } else if ret.ends_with("t") {
            // Hubert, Zygmunt
            ret.truncate(ret.len() - 1);
            ret.push_str("cie");
        } else if ret.ends_with("i") || ret.ends_with("y") {
            // do nothing
            // Ignacy -> Ignacy, Antoni -> Antoni
        }
         else {
            // generyczne dodanie u, Lukasz(u), Grzegorz(u)
            ret.push_str("u")
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use crate::Osoba;
    use crate::Deklinacja;
    #[test]
    fn imiona_meskie() {
        assert_eq!(Osoba { first_name: "Adam".to_string()}.wolacz(), "Adamie".to_string());
        assert_eq!(Osoba { first_name: "Albert".to_string()}.wolacz(), "Albercie".to_string());
        assert_eq!(Osoba { first_name: "Andrzej".to_string()}.wolacz(), "Andrzeju".to_string());
        assert_eq!(Osoba { first_name: "Antoni".to_string()}.wolacz(), "Antoni".to_string());
        assert_eq!(Osoba { first_name: "Antek".to_string()}.wolacz(), "Antku".to_string());
        assert_eq!(Osoba { first_name: "Bartek".to_string()}.wolacz(), "Bartku".to_string());
        assert_eq!(Osoba { first_name: "Bartlomiej".to_string()}.wolacz(), "Bartlomieju".to_string());
        assert_eq!(Osoba { first_name: "Damian".to_string()}.wolacz(), "Damianie".to_string());
        assert_eq!(Osoba { first_name: "Daniel".to_string()}.wolacz(), "Danielu".to_string());
        assert_eq!(Osoba { first_name: "Filip".to_string()}.wolacz(), "Filipie".to_string());
        assert_eq!(Osoba { first_name: "Franciszek".to_string()}.wolacz(), "Franciszku".to_string());
        assert_eq!(Osoba { first_name: "Grzegorz".to_string()}.wolacz(), "Grzegorzu".to_string());
        assert_eq!(Osoba { first_name: "Hubert".to_string()}.wolacz(), "Hubercie".to_string());
        assert_eq!(Osoba { first_name: "Ignacy".to_string()}.wolacz(), "Ignacy".to_string());
        assert_eq!(Osoba { first_name: "Jakub".to_string()}.wolacz(), "Jakubie".to_string());
        assert_eq!(Osoba { first_name: "Jan".to_string()}.wolacz(), "Janie".to_string());
        assert_eq!(Osoba { first_name: "Kacper".to_string()}.wolacz(), "Kacprze".to_string());
        assert_eq!(Osoba { first_name: "Kazimierz".to_string()}.wolacz(), "Kazimierzu".to_string());
        assert_eq!(Osoba { first_name: "Lukasz".to_string()}.wolacz(), "Lukaszu".to_string());
        assert_eq!(Osoba { first_name: "Maciej".to_string()}.wolacz(), "Macieju".to_string());
        assert_eq!(Osoba { first_name: "Marek".to_string()}.wolacz(), "Marku".to_string());
        assert_eq!(Osoba { first_name: "Michal".to_string()}.wolacz(), "Michale".to_string());
        assert_eq!(Osoba { first_name: "Nikodem".to_string()}.wolacz(), "Nikodemie".to_string());
        assert_eq!(Osoba { first_name: "Pawel".to_string()}.wolacz(), "Pawle".to_string());
        assert_eq!(Osoba { first_name: "Piotr".to_string()}.wolacz(), "Piotrze".to_string());
        assert_eq!(Osoba { first_name: "Przemek".to_string()}.wolacz(), "Przemku".to_string());
        assert_eq!(Osoba { first_name: "Rafal".to_string()}.wolacz(), "Rafale".to_string());
        assert_eq!(Osoba { first_name: "Stanislaw".to_string()}.wolacz(), "Stanislawie".to_string());
        assert_eq!(Osoba { first_name: "Szymon".to_string()}.wolacz(), "Szymonie".to_string());
        assert_eq!(Osoba { first_name: "Tomek".to_string()}.wolacz(), "Tomku".to_string());   
        assert_eq!(Osoba { first_name: "Waldemar".to_string()}.wolacz(), "Waldemarze".to_string());
        assert_eq!(Osoba { first_name: "Wieslaw".to_string()}.wolacz(), "Wieslawie".to_string());
        assert_eq!(Osoba { first_name: "Wiktor".to_string()}.wolacz(), "Wiktorze".to_string());
        assert_eq!(Osoba { first_name: "Zbigniew".to_string()}.wolacz(), "Zbigniewie".to_string());
        assert_eq!(Osoba { first_name: "Zygmunt".to_string()}.wolacz(), "Zygmuncie".to_string());
    }
    #[test]
    fn imiona_zenskie() {
        assert_eq!(Osoba { first_name: "Aleksandra".to_string()}.wolacz(), "Aleksandro".to_string());
        assert_eq!(Osoba { first_name: "Alicja".to_string()}.wolacz(), "Alicjo".to_string());
        assert_eq!(Osoba { first_name: "Karolina".to_string()}.wolacz(), "Karolino".to_string());
        assert_eq!(Osoba { first_name: "Monika".to_string()}.wolacz(), "Moniko".to_string());
        assert_eq!(Osoba { first_name: "Wanda".to_string()}.wolacz(), "Wando".to_string());

    }
}