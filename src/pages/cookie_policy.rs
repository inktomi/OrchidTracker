use leptos::prelude::*;

const SECTION_HEADING: &str = "mt-8 mb-3 text-xl text-stone-800 dark:text-stone-100";
const PARAGRAPH: &str = "mb-4 text-sm leading-relaxed text-stone-600 dark:text-stone-300";
const TABLE_HEADER: &str = "py-2.5 px-3 text-left text-xs font-semibold tracking-wider uppercase border-b text-stone-500 border-stone-200 bg-stone-50 dark:text-stone-400 dark:border-stone-700 dark:bg-stone-800/50";
const TABLE_CELL: &str = "py-2.5 px-3 text-sm border-b text-stone-600 border-stone-100 dark:text-stone-300 dark:border-stone-800";

#[component]
pub fn CookiePolicyPage() -> impl IntoView {
    view! {
        <main class="min-h-screen bg-cream">
            <div class="py-12 px-6 mx-auto max-w-2xl sm:px-8">
                // Header
                <div class="mb-8">
                    <a href="/" class="inline-flex gap-1 items-center mb-6 text-sm transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" viewBox="0 0 20 20" fill="currentColor">
                            <path fill-rule="evenodd" d="M9.707 16.707a1 1 0 01-1.414 0l-6-6a1 1 0 010-1.414l6-6a1 1 0 011.414 1.414L5.414 9H17a1 1 0 110 2H5.414l4.293 4.293a1 1 0 010 1.414z" clip-rule="evenodd" />
                        </svg>
                        "Back to Velamen"
                    </a>
                    <h1 class="mb-2 text-3xl text-stone-800 dark:text-stone-100">"Cookie Policy"</h1>
                    <p class="text-sm text-stone-500 dark:text-stone-400">"Last updated: February 27, 2026"</p>
                </div>

                // Introduction
                <p class=PARAGRAPH>
                    "Velamen (\"we\", \"us\", or \"our\") uses cookies on the velamen.app website (the \"Service\"). This Cookie Policy explains what cookies are, what cookies we use, and how you can manage them."
                </p>

                // What are cookies
                <h2 class=SECTION_HEADING>"What Are Cookies?"</h2>
                <p class=PARAGRAPH>
                    "Cookies are small text files that are stored on your device (computer, tablet, or mobile phone) when you visit a website. They are widely used to make websites work efficiently and to provide information to the site owners."
                </p>

                // What cookies we use
                <h2 class=SECTION_HEADING>"Cookies We Use"</h2>
                <p class=PARAGRAPH>
                    "We use only strictly necessary (essential) cookies. We do not use any analytics, advertising, tracking, or preference cookies. The following table details every cookie set by our Service:"
                </p>

                <div class="overflow-x-auto mb-6 rounded-lg border border-stone-200 dark:border-stone-700">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr>
                                <th class=TABLE_HEADER>"Cookie Name"</th>
                                <th class=TABLE_HEADER>"Provider"</th>
                                <th class=TABLE_HEADER>"Purpose"</th>
                                <th class=TABLE_HEADER>"Duration"</th>
                                <th class=TABLE_HEADER>"Type"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td class=TABLE_CELL>
                                    <code class="py-0.5 px-1.5 text-xs rounded bg-stone-100 text-stone-700 dark:bg-stone-800 dark:text-stone-300">"id"</code>
                                </td>
                                <td class=TABLE_CELL>"velamen.app"</td>
                                <td class=TABLE_CELL>"Maintains your authenticated session so you stay logged in as you navigate between pages. Contains a random session identifier only\u{2014}no personal data is stored in the cookie itself."</td>
                                <td class=TABLE_CELL>"7 days of inactivity"</td>
                                <td class=TABLE_CELL>"Essential"</td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                // Cookie attributes
                <h2 class=SECTION_HEADING>"Cookie Security"</h2>
                <p class=PARAGRAPH>
                    "Our session cookie is configured with the following security attributes:"
                </p>
                <ul class="mb-6 ml-6 space-y-1.5 text-sm leading-relaxed list-disc text-stone-600 dark:text-stone-300">
                    <li><strong>"HttpOnly"</strong>"\u{2014}The cookie cannot be accessed by JavaScript, protecting against cross-site scripting (XSS) attacks."</li>
                    <li><strong>"Secure"</strong>"\u{2014}The cookie is only transmitted over encrypted HTTPS connections."</li>
                    <li><strong>"SameSite=Strict"</strong>"\u{2014}The cookie is never sent with cross-site requests, protecting against cross-site request forgery (CSRF) attacks."</li>
                </ul>

                // What data is collected
                <h2 class=SECTION_HEADING>"What Personal Data Do Cookies Collect?"</h2>
                <p class=PARAGRAPH>
                    "The session cookie itself contains only a randomly generated session identifier. It does not contain your username, email, password, or any other personal information. On our server, this identifier is linked to your user account to maintain your login state. Session data is stored in our database and is automatically deleted when it expires."
                </p>

                // Third parties
                <h2 class=SECTION_HEADING>"Third-Party Cookies"</h2>
                <p class=PARAGRAPH>
                    "We do not set any third-party cookies. We do not use any analytics services, advertising networks, or social media tracking pixels."
                </p>
                <p class=PARAGRAPH>
                    "Our Service loads fonts from Google Fonts (fonts.googleapis.com). Google Fonts does not set cookies, but Google may log your IP address when fonts are requested. Refer to "
                    <a href="https://developers.google.com/fonts/faq/privacy" target="_blank" rel="noopener noreferrer" class="font-medium underline transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">"Google\u{2019}s privacy policy"</a>
                    " for details."
                </p>

                // Local storage
                <h2 class=SECTION_HEADING>"Local Storage"</h2>
                <p class=PARAGRAPH>
                    "In addition to cookies, we use your browser\u{2019}s local storage to remember your dark/light theme preference. Local storage is not a cookie\u{2014}it is not sent to our servers with requests and is only accessible by our website on your device."
                </p>

                <div class="overflow-x-auto mb-6 rounded-lg border border-stone-200 dark:border-stone-700">
                    <table class="w-full text-left border-collapse">
                        <thead>
                            <tr>
                                <th class=TABLE_HEADER>"Key"</th>
                                <th class=TABLE_HEADER>"Purpose"</th>
                                <th class=TABLE_HEADER>"Duration"</th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr>
                                <td class=TABLE_CELL>
                                    <code class="py-0.5 px-1.5 text-xs rounded bg-stone-100 text-stone-700 dark:bg-stone-800 dark:text-stone-300">"dark_mode"</code>
                                </td>
                                <td class=TABLE_CELL>"Remembers your light or dark theme preference to avoid a flash of the wrong theme on page load."</td>
                                <td class=TABLE_CELL>"Persistent (until cleared)"</td>
                            </tr>
                            <tr>
                                <td class=TABLE_CELL>
                                    <code class="py-0.5 px-1.5 text-xs rounded bg-stone-100 text-stone-700 dark:bg-stone-800 dark:text-stone-300">"velamen_cookie_consent"</code>
                                </td>
                                <td class=TABLE_CELL>"Records that you have acknowledged our cookie notice, so it is not shown again."</td>
                                <td class=TABLE_CELL>"Persistent (until cleared)"</td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                // Managing cookies
                <h2 class=SECTION_HEADING>"How to Manage Cookies"</h2>
                <p class=PARAGRAPH>
                    "Because we only use essential cookies that are strictly necessary for the Service to function, there are no optional cookies to accept or reject. If you disable cookies in your browser, you will not be able to log in or use authenticated features of the Service."
                </p>
                <p class=PARAGRAPH>
                    "You can delete existing cookies and configure your browser to block cookies through your browser settings:"
                </p>
                <ul class="mb-6 ml-6 space-y-1.5 text-sm leading-relaxed list-disc text-stone-600 dark:text-stone-300">
                    <li><strong>"Chrome"</strong>": Settings \u{2192} Privacy and security \u{2192} Cookies and other site data"</li>
                    <li><strong>"Firefox"</strong>": Settings \u{2192} Privacy & Security \u{2192} Cookies and Site Data"</li>
                    <li><strong>"Safari"</strong>": Preferences \u{2192} Privacy \u{2192} Manage Website Data"</li>
                    <li><strong>"Edge"</strong>": Settings \u{2192} Cookies and site permissions \u{2192} Manage and delete cookies"</li>
                </ul>

                // Data sharing
                <h2 class=SECTION_HEADING>"Who Has Access to Cookie Data?"</h2>
                <p class=PARAGRAPH>
                    "Session data associated with your cookie is stored on our self-hosted server and is not shared with any third parties. We do not sell, trade, or transfer cookie data to outside parties."
                </p>

                // Legal basis
                <h2 class=SECTION_HEADING>"Legal Basis"</h2>
                <p class=PARAGRAPH>
                    "Our session cookie is classified as a strictly necessary cookie under the EU ePrivacy Directive (Article 5(3)) and GDPR (Article 6(1)(f)). Strictly necessary cookies are exempt from the requirement to obtain consent because the Service cannot function without them. We provide this policy for transparency."
                </p>
                <p class=PARAGRAPH>
                    "Under the California Consumer Privacy Act (CCPA), you have the right to know what personal information is collected. Our session cookie collects only a random session identifier as described above."
                </p>

                // Changes
                <h2 class=SECTION_HEADING>"Changes to This Policy"</h2>
                <p class=PARAGRAPH>
                    "We may update this Cookie Policy from time to time. If we add non-essential cookies in the future, we will update this policy and request your consent before setting them. The \"Last updated\" date at the top of this page indicates when it was last revised."
                </p>

                // Contact
                <h2 class=SECTION_HEADING>"Contact"</h2>
                <p class=PARAGRAPH>
                    "If you have questions about this Cookie Policy, please contact us through the Velamen application."
                </p>

                // Footer
                <div class="pt-6 mt-8 text-xs border-t border-stone-200 text-stone-400 dark:border-stone-700 dark:text-stone-500">
                    "\u{00a9} 2026 Velamen. All rights reserved."
                </div>
            </div>
        </main>
    }
}
