use leptos::prelude::*;

// ── Full Phalaenopsis Orchid Spray ──────────────────────────────────────
// Arching flower spike with 4 open blooms, buds, and broad leaves at the base.
// Designed as a large background decoration for auth page left panels.
// viewBox 0 0 420 730 — tall vertical composition.

#[component]
pub fn OrchidSpray(
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 420 730"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            // ── Leaves at base ──
            <g class="bo-delay-0">
                // Left leaf — broad, arching
                <path d="M 196,698 C 182,678 148,656 112,665 C 82,673 62,695 75,710 C 88,722 152,712 182,700 Z" stroke-width="1.2" pathLength="1" />
                // Left leaf midrib
                <path d="M 190,695 C 172,680 138,668 98,677" stroke-width="0.6" pathLength="1" />
                // Left leaf secondary veins
                <path d="M 180,692 C 165,682 140,676 112,682" stroke-width="0.35" pathLength="1" />
                <path d="M 170,702 C 155,696 135,694 108,698" stroke-width="0.35" pathLength="1" />

                // Right leaf — broad, arching
                <path d="M 204,698 C 218,678 252,656 288,665 C 318,673 338,695 325,710 C 312,722 248,712 218,700 Z" stroke-width="1.2" pathLength="1" />
                // Right leaf midrib
                <path d="M 210,695 C 228,680 262,668 302,677" stroke-width="0.6" pathLength="1" />
                // Right leaf secondary veins
                <path d="M 220,692 C 235,682 260,676 288,682" stroke-width="0.35" pathLength="1" />
                <path d="M 230,702 C 245,696 265,694 292,698" stroke-width="0.35" pathLength="1" />

                // Small center leaf
                <path d="M 197,692 C 194,672 191,645 193,620 C 196,645 200,672 203,692" stroke-width="1" pathLength="1" />
                <path d="M 200,690 C 197,668 194,642 196,622" stroke-width="0.5" pathLength="1" />
            </g>

            // ── Main flower spike (stem) ──
            <g class="bo-delay-1">
                <path d="M 200,690 C 198,640 192,580 188,520 C 184,460 186,400 194,350 C 202,300 218,255 232,210 C 246,165 250,125 244,90 C 240,68 235,55 230,48" stroke-width="1.8" pathLength="1" />
            </g>

            // ── Branch stems ──
            <g class="bo-delay-1">
                <path d="M 188,520 C 172,512 155,508 138,512" stroke-width="1.3" pathLength="1" />
                <path d="M 190,430 C 206,420 224,414 245,418" stroke-width="1.3" pathLength="1" />
                <path d="M 198,345 C 180,336 162,332 145,338" stroke-width="1.3" pathLength="1" />
                <path d="M 222,260 C 238,250 256,245 272,250" stroke-width="1.3" pathLength="1" />
                <path d="M 240,175 C 250,168 260,165 268,168" stroke-width="1.1" pathLength="1" />
                <path d="M 244,118 C 252,112 260,108 266,112" stroke-width="1" pathLength="1" />
            </g>

            // ── Bloom 1 — bottom left, centered ~(125, 520) ──
            <g class="bo-delay-2">
                // Dorsal sepal
                <path d="M 125,492 C 115,490 108,498 109,508 C 110,518 118,526 125,528 C 132,526 140,518 141,508 C 142,498 135,490 125,492" stroke-width="0.9" pathLength="1" />
                // Dorsal vein
                <path d="M 125,494 C 125,502 125,516 125,526" stroke-width="0.35" pathLength="1" />
                // Left lateral sepal
                <path d="M 121,527 C 114,533 102,540 94,538 C 88,536 90,528 96,523 C 102,518 114,520 121,527" stroke-width="0.9" pathLength="1" />
                // Right lateral sepal
                <path d="M 129,527 C 136,533 148,540 156,538 C 162,536 160,528 154,523 C 148,518 136,520 129,527" stroke-width="0.9" pathLength="1" />
                // Left petal
                <path d="M 123,510 C 116,504 104,500 100,506 C 97,512 104,522 114,523 C 122,524 124,518 123,510" stroke-width="0.8" pathLength="1" />
                // Right petal
                <path d="M 127,510 C 134,504 146,500 150,506 C 153,512 146,522 136,523 C 128,524 126,518 127,510" stroke-width="0.8" pathLength="1" />
                // Lip
                <path d="M 125,528 C 120,533 114,542 116,548 C 118,553 122,555 125,553 C 128,555 132,553 134,548 C 136,542 130,533 125,528" stroke-width="0.8" pathLength="1" />
                // Column
                <path d="M 125,518 C 123,522 124,527 125,528 C 126,527 127,522 125,518" stroke-width="0.5" pathLength="1" />
                // Sepal veins (subtle)
                <path d="M 98,532 L 120,527" stroke-width="0.3" pathLength="1" />
                <path d="M 152,532 L 130,527" stroke-width="0.3" pathLength="1" />
            </g>

            // ── Bloom 2 — right side, centered ~(255, 418) ──
            <g class="bo-delay-3">
                // Dorsal sepal
                <path d="M 255,390 C 245,388 238,396 239,406 C 240,416 248,424 255,426 C 262,424 270,416 271,406 C 272,396 265,388 255,390" stroke-width="0.9" pathLength="1" />
                <path d="M 255,392 C 255,400 255,414 255,424" stroke-width="0.35" pathLength="1" />
                // Left lateral sepal
                <path d="M 251,425 C 244,431 232,438 224,436 C 218,434 220,426 226,421 C 232,416 244,418 251,425" stroke-width="0.9" pathLength="1" />
                // Right lateral sepal
                <path d="M 259,425 C 266,431 278,438 286,436 C 292,434 290,426 284,421 C 278,416 266,418 259,425" stroke-width="0.9" pathLength="1" />
                // Left petal
                <path d="M 253,408 C 246,402 234,398 230,404 C 227,410 234,420 244,421 C 252,422 254,416 253,408" stroke-width="0.8" pathLength="1" />
                // Right petal
                <path d="M 257,408 C 264,402 276,398 280,404 C 283,410 276,420 266,421 C 258,422 256,416 257,408" stroke-width="0.8" pathLength="1" />
                // Lip
                <path d="M 255,426 C 250,431 244,440 246,446 C 248,451 252,453 255,451 C 258,453 262,451 264,446 C 266,440 260,431 255,426" stroke-width="0.8" pathLength="1" />
                // Column
                <path d="M 255,416 C 253,420 254,425 255,426 C 256,425 257,420 255,416" stroke-width="0.5" pathLength="1" />
                <path d="M 228,432 L 250,425" stroke-width="0.3" pathLength="1" />
                <path d="M 282,432 L 260,425" stroke-width="0.3" pathLength="1" />
            </g>

            // ── Bloom 3 — left side, centered ~(135, 338) ──
            <g class="bo-delay-4">
                // Dorsal sepal
                <path d="M 135,310 C 125,308 118,316 119,326 C 120,336 128,344 135,346 C 142,344 150,336 151,326 C 152,316 145,308 135,310" stroke-width="0.9" pathLength="1" />
                <path d="M 135,312 C 135,320 135,334 135,344" stroke-width="0.35" pathLength="1" />
                // Left lateral sepal
                <path d="M 131,345 C 124,351 112,358 104,356 C 98,354 100,346 106,341 C 112,336 124,338 131,345" stroke-width="0.9" pathLength="1" />
                // Right lateral sepal
                <path d="M 139,345 C 146,351 158,358 166,356 C 172,354 170,346 164,341 C 158,336 146,338 139,345" stroke-width="0.9" pathLength="1" />
                // Left petal
                <path d="M 133,328 C 126,322 114,318 110,324 C 107,330 114,340 124,341 C 132,342 134,336 133,328" stroke-width="0.8" pathLength="1" />
                // Right petal
                <path d="M 137,328 C 144,322 156,318 160,324 C 163,330 156,340 146,341 C 138,342 136,336 137,328" stroke-width="0.8" pathLength="1" />
                // Lip
                <path d="M 135,346 C 130,351 124,360 126,366 C 128,371 132,373 135,371 C 138,373 142,371 144,366 C 146,360 140,351 135,346" stroke-width="0.8" pathLength="1" />
                // Column
                <path d="M 135,336 C 133,340 134,345 135,346 C 136,345 137,340 135,336" stroke-width="0.5" pathLength="1" />
                <path d="M 108,350 L 130,345" stroke-width="0.3" pathLength="1" />
                <path d="M 162,350 L 140,345" stroke-width="0.3" pathLength="1" />
            </g>

            // ── Bloom 4 — right, slightly smaller, centered ~(278, 252) ──
            <g class="bo-delay-5">
                // Dorsal sepal (tighter)
                <path d="M 278,228 C 270,226 264,233 265,241 C 266,249 272,256 278,258 C 284,256 290,249 291,241 C 292,233 286,226 278,228" stroke-width="0.85" pathLength="1" />
                <path d="M 278,230 C 278,238 278,250 278,256" stroke-width="0.3" pathLength="1" />
                // Left lateral sepal
                <path d="M 275,257 C 269,262 260,268 254,266 C 249,264 251,258 256,254 C 260,250 270,251 275,257" stroke-width="0.85" pathLength="1" />
                // Right lateral sepal
                <path d="M 281,257 C 287,262 296,268 302,266 C 307,264 305,258 300,254 C 296,250 286,251 281,257" stroke-width="0.85" pathLength="1" />
                // Left petal
                <path d="M 276,243 C 270,238 262,235 259,240 C 257,245 262,253 270,254 C 276,254 277,248 276,243" stroke-width="0.75" pathLength="1" />
                // Right petal
                <path d="M 280,243 C 286,238 294,235 297,240 C 299,245 294,253 286,254 C 280,254 279,248 280,243" stroke-width="0.75" pathLength="1" />
                // Lip
                <path d="M 278,258 C 274,262 270,269 272,274 C 274,278 276,279 278,278 C 280,279 282,278 284,274 C 286,269 282,262 278,258" stroke-width="0.75" pathLength="1" />
                // Column
                <path d="M 278,250 C 276,253 277,257 278,258 C 279,257 280,253 278,250" stroke-width="0.45" pathLength="1" />
            </g>

            // ── Bud near top ~(268, 168) ──
            <g class="bo-delay-6">
                <path d="M 268,158 C 264,161 261,167 263,174 C 265,180 270,182 273,178 C 276,174 275,166 272,160 C 270,158 268,158 268,158" stroke-width="0.9" pathLength="1" />
                <path d="M 268,160 C 268,166 267,173 266,178" stroke-width="0.4" pathLength="1" />
            </g>

            // ── Small bud at tip ~(266, 112) ──
            <g class="bo-delay-6">
                <path d="M 266,104 C 263,107 261,112 263,117 C 265,121 268,122 270,119 C 272,116 271,109 268,105 Z" stroke-width="0.8" pathLength="1" />
            </g>

            // ── Tiny bud at very top ~(234, 52) ──
            <g class="bo-delay-6">
                <path d="M 234,44 C 231,47 230,52 232,56 C 234,59 237,59 238,56 C 239,52 237,47 234,44" stroke-width="0.7" pathLength="1" />
            </g>

            // ── Decorative stipple dots along stem (vintage engraving detail) ──
            <g class="bo-delay-1" stroke-width="0">
                <circle cx="196" cy="600" r="0.8" fill="currentColor" />
                <circle cx="194" cy="570" r="0.6" fill="currentColor" />
                <circle cx="190" cy="550" r="0.7" fill="currentColor" />
                <circle cx="186" cy="490" r="0.6" fill="currentColor" />
                <circle cx="186" cy="470" r="0.7" fill="currentColor" />
                <circle cx="188" cy="410" r="0.6" fill="currentColor" />
                <circle cx="192" cy="380" r="0.7" fill="currentColor" />
                <circle cx="200" cy="330" r="0.6" fill="currentColor" />
                <circle cx="210" cy="290" r="0.6" fill="currentColor" />
                <circle cx="224" cy="240" r="0.5" fill="currentColor" />
                <circle cx="236" cy="200" r="0.5" fill="currentColor" />
                <circle cx="244" cy="155" r="0.5" fill="currentColor" />
                <circle cx="244" cy="100" r="0.4" fill="currentColor" />
            </g>
        </svg>
    }.into_any()
}


// ── Single Orchid Bloom with Short Stem ────────────────────────────────
// A smaller decorative piece — one detailed bloom on a curved stem with a leaf.
// Used as a subtle background accent on the home/main page.
// viewBox 0 0 200 260

#[component]
pub fn OrchidAccent(
    #[prop(default = "")] class: &'static str,
) -> impl IntoView {
    view! {
        <svg
            viewBox="0 0 200 260"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            stroke="currentColor"
            stroke-linecap="round"
            stroke-linejoin="round"
            class=class
        >
            // ── Leaf ──
            <path d="M 95,250 C 85,235 60,218 38,224 C 20,230 14,248 22,256 C 32,262 72,256 88,248 Z" stroke-width="1.1" pathLength="1" />
            <path d="M 90,248 C 76,237 52,228 30,234" stroke-width="0.5" pathLength="1" />

            // ── Stem ──
            <path d="M 98,248 C 96,220 94,190 96,165 C 98,140 104,118 112,100 C 120,82 126,68 126,55" stroke-width="1.5" pathLength="1" />

            // ── Branch to bloom ──
            <path d="M 112,100 C 122,94 134,90 146,94" stroke-width="1.2" pathLength="1" />

            // ── Bloom centered ~(155, 95) ──
            // Dorsal sepal
            <path d="M 155,68 C 146,66 140,74 141,83 C 142,92 149,99 155,101 C 161,99 168,92 169,83 C 170,74 164,66 155,68" stroke-width="0.9" pathLength="1" />
            <path d="M 155,70 C 155,78 155,92 155,99" stroke-width="0.35" pathLength="1" />
            // Left lateral sepal
            <path d="M 151,100 C 145,106 134,112 127,110 C 122,108 124,101 129,96 C 134,92 146,93 151,100" stroke-width="0.9" pathLength="1" />
            // Right lateral sepal
            <path d="M 159,100 C 165,106 176,112 183,110 C 188,108 186,101 181,96 C 176,92 164,93 159,100" stroke-width="0.9" pathLength="1" />
            // Left petal
            <path d="M 153,85 C 147,80 138,77 135,82 C 132,87 138,96 146,97 C 152,97 154,92 153,85" stroke-width="0.8" pathLength="1" />
            // Right petal
            <path d="M 157,85 C 163,80 172,77 175,82 C 178,87 172,96 164,97 C 158,97 156,92 157,85" stroke-width="0.8" pathLength="1" />
            // Lip
            <path d="M 155,101 C 150,106 146,114 148,120 C 150,124 153,126 155,124 C 157,126 160,124 162,120 C 164,114 160,106 155,101" stroke-width="0.8" pathLength="1" />
            // Column
            <path d="M 155,92 C 153,96 154,100 155,101 C 156,100 157,96 155,92" stroke-width="0.5" pathLength="1" />

            // ── Upper bud ──
            <path d="M 126,55 C 132,48 138,44 142,46" stroke-width="1" pathLength="1" />
            <path d="M 142,38 C 139,42 138,47 140,52 C 142,55 145,56 146,53 C 148,50 146,44 143,40 Z" stroke-width="0.8" pathLength="1" />

            // Stipple dots
            <g stroke-width="0">
                <circle cx="97" cy="230" r="0.6" fill="currentColor" />
                <circle cx="95" cy="205" r="0.5" fill="currentColor" />
                <circle cx="95" cy="180" r="0.5" fill="currentColor" />
                <circle cx="98" cy="150" r="0.5" fill="currentColor" />
                <circle cx="106" cy="115" r="0.5" fill="currentColor" />
                <circle cx="118" cy="90" r="0.4" fill="currentColor" />
            </g>
        </svg>
    }.into_any()
}
