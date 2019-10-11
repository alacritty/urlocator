macro_rules! schemes {
    ($name:ident {
        $([$state:ident, $($match:literal)|+ => $result:ident]$(,)?)*
    }
    [$($complete:ident),*]) => (
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        pub enum $name {
            $($result,)*
            COMPLETE,
            NONE,
        }

        impl Default for $name {
            #[inline]
            fn default() -> Self {
                Self::NONE
            }
        }

        impl $name {
            #[inline]
            pub fn advance(self, c: char) -> Self {
                match (self, c) {
                    $($((Self::$state, $match))|+ => Self::$result,)*
                    $((Self::$complete, ':') => Self::COMPLETE,)*
                    _ => Self::NONE,
                }
            }
        }
    )
}

schemes! {
    SchemeState {
        [NONE, 'h'|'H' => H],
        [H, 't'|'T' => HT],
        [HT, 't'|'T' => HTT],
        [HTT, 'p'|'P' => HTTP],
        [HTTP, 's'|'S' => HTTPS],
        [NONE, 'f'|'F' => F],
        [F, 't'|'T' => FT],
        [FT, 'p'|'P' => FTP],
        [F, 'i'|'I' => FI],
        [FI, 'l'|'L' => FIL],
        [FIL, 'e'|'E' => FILE],
        [NONE, 'm'|'M' => M],
        [M, 'a'|'A' => MA],
        [MA, 'i'|'I' => MAI],
        [MAI, 'l'|'L' => MAIL],
        [MAIL, 't'|'T' => MAILT],
        [MAILT, 'o'|'O' => MAILTO],
        [NONE, 'n'|'N' => N],
        [N, 'e'|'E' => NE],
        [NE, 'w'|'W' => NEW],
        [NEW, 's'|'S' => NEWS],
        [NONE, 'g'|'G' => G],
        [G, 'i'|'I' => GI],
        [GI, 't'|'T' => GIT],
        [NONE, 's'|'S' => S],
        [S, 's'|'S' => SS],
        [SS, 'h'|'H' => SSH],
    }

    [HTTP, HTTPS, FTP, FILE, MAILTO, NEWS, GIT, SSH]
}
