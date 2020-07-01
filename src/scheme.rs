macro_rules! schemes {
    ($name:ident {
        $([$state:ident, $($match:literal)|+ => $result:ident]$(,)?)*
    }
    [$($complete:ident),*]) => (
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($result,)*
            COMPLETE,
            INVALID,
            RESET,
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                $name::RESET
            }
        }

        impl $name {
            #[inline]
            pub fn advance(self, c: char) -> Self {
                match (self, c) {
                    $($(($name::$state, $match))|+ => $name::$result,)*
                    $(($name::$complete, ':') => $name::COMPLETE,)*
                    (_, 'a'..='z') | (_, 'A'..='Z') => $name::INVALID,
                    _ => $name::RESET,
                }
            }
        }
    )
}

schemes! {
    SchemeState {
        [RESET, 'h'|'H' => H],
        [H, 't'|'T' => HT],
        [HT, 't'|'T' => HTT],
        [HTT, 'p'|'P' => HTTP],
        [HTTP, 's'|'S' => HTTPS],
        [RESET, 'f'|'F' => F],
        [F, 't'|'T' => FT],
        [FT, 'p'|'P' => FTP],
        [F, 'i'|'I' => FI],
        [FI, 'l'|'L' => FIL],
        [FIL, 'e'|'E' => FILE],
        [RESET, 'm'|'M' => M],
        [M, 'a'|'A' => MA],
        [MA, 'i'|'I' => MAI],
        [MAI, 'l'|'L' => MAIL],
        [MAIL, 't'|'T' => MAILT],
        [MAILT, 'o'|'O' => MAILTO],
        [RESET, 'n'|'N' => N],
        [N, 'e'|'E' => NE],
        [NE, 'w'|'W' => NEW],
        [NEW, 's'|'S' => NEWS],
        [RESET, 'g'|'G' => G],
        [G, 'e'|'E' => GE],
        [GE, 'm'|'M' => GEM],
        [GEM, 'i'|'I' => GEMI],
        [GEMI, 'n'|'N' => GEMIN],
        [GEMIN, 'i'|'I' => GEMINI],
        [G, 'i'|'I' => GI],
        [GI, 't'|'T' => GIT],
        [G, 'o'|'O' => GO],
        [GO, 'p'|'P' => GOP],
        [GOP, 'h'|'H' => GOPH],
        [GOPH, 'e'|'E' => GOPHE],
        [GOPHE, 'r'|'R' => GOPHER],
        [RESET, 's'|'S' => S],
        [S, 's'|'S' => SS],
        [SS, 'h'|'H' => SSH],
    }

    [HTTP, HTTPS, FTP, FILE, MAILTO, NEWS, GEMINI, GIT, GOPHER, SSH]
}
