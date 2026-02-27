use leptos::prelude::*;

const SECTION_HEADING: &str = "mt-8 mb-3 text-xl text-stone-800 dark:text-stone-100";
const PARAGRAPH: &str = "mb-4 text-sm leading-relaxed text-stone-600 dark:text-stone-300";

#[component]
pub fn TermsOfServicePage() -> impl IntoView {
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
                    <h1 class="mb-2 text-3xl text-stone-800 dark:text-stone-100">"Terms of Service"</h1>
                    <p class="text-sm text-stone-500 dark:text-stone-400">"Last updated: February 27, 2026"</p>
                </div>

                // Introduction
                <p class=PARAGRAPH>
                    "Welcome to Velamen (\"we\", \"us\", or \"our\"). By accessing or using the Velamen application at velamen.app (the \"Service\"), you agree to be bound by these Terms of Service (\"Terms\"). If you do not agree to these Terms, do not use the Service."
                </p>

                // 1. Description of Service
                <h2 class=SECTION_HEADING>"1. Description of Service"</h2>
                <p class=PARAGRAPH>
                    "Velamen is a self-hosted plant collection management application that allows users to track orchids and other plants, record care history, monitor growing conditions, upload photographs, and receive AI-assisted species identification and care recommendations. The Service is provided on an \"as is\" and \"as available\" basis."
                </p>

                // 2. User Accounts
                <h2 class=SECTION_HEADING>"2. User Accounts"</h2>
                <p class=PARAGRAPH>
                    "To use the Service, you must create an account by providing a username, email address, and password. You are responsible for:"
                </p>
                <ul class="mb-4 ml-6 space-y-1.5 text-sm leading-relaxed list-disc text-stone-600 dark:text-stone-300">
                    <li>"Maintaining the confidentiality of your login credentials"</li>
                    <li>"All activities that occur under your account"</li>
                    <li>"Notifying the administrator promptly if you suspect unauthorized access to your account"</li>
                </ul>
                <p class=PARAGRAPH>
                    "We reserve the right to suspend or terminate accounts that violate these Terms."
                </p>

                // 3. Permitted Use
                <h2 class=SECTION_HEADING>"3. Permitted Use"</h2>
                <p class=PARAGRAPH>
                    "You may use the Service for lawful, personal, non-commercial purposes related to managing your plant collection. You are granted a limited, non-exclusive, non-transferable, revocable license to access and use the Service subject to these Terms."
                </p>

                // 4. Prohibited Use
                <h2 class=SECTION_HEADING>"4. Prohibited Use"</h2>
                <p class=PARAGRAPH>
                    "When using the Service, you agree not to:"
                </p>
                <ul class="mb-4 ml-6 space-y-1.5 text-sm leading-relaxed list-disc text-stone-600 dark:text-stone-300">
                    <li>"Violate any applicable local, state, national, or international law or regulation"</li>
                    <li>"Upload content that is unlawful, harmful, threatening, abusive, defamatory, or otherwise objectionable"</li>
                    <li>"Attempt to gain unauthorized access to any portion of the Service, other user accounts, or any systems or networks connected to the Service"</li>
                    <li>"Use the Service to transmit viruses, malware, or other malicious code"</li>
                    <li>"Interfere with or disrupt the integrity or performance of the Service"</li>
                    <li>"Scrape, crawl, or use automated means to access the Service without prior written consent"</li>
                    <li>"Impersonate any person or entity, or falsely represent your affiliation with any person or entity"</li>
                    <li>"Use the AI identification features to make decisions regarding protected or endangered species that require professional botanical or legal expertise"</li>
                </ul>

                // 5. User-Generated Content
                <h2 class=SECTION_HEADING>"5. User-Generated Content"</h2>
                <p class=PARAGRAPH>
                    "You retain ownership of all content you upload to the Service, including photographs, care notes, and plant data (\"User Content\"). By uploading User Content, you grant us a limited license to store and process that content solely for the purpose of providing the Service to you."
                </p>
                <p class=PARAGRAPH>
                    "You are solely responsible for your User Content. You represent and warrant that you own or have the necessary rights to upload any content you submit, and that your content does not infringe the intellectual property or other rights of any third party."
                </p>

                // 6. AI Features Disclaimer
                <h2 class=SECTION_HEADING>"6. AI Features Disclaimer"</h2>
                <p class=PARAGRAPH>
                    "The Service includes AI-powered features for plant identification and care recommendations. These features are provided for informational purposes only and should not be relied upon as professional botanical advice. AI-generated identifications and recommendations may be inaccurate, incomplete, or outdated."
                </p>
                <p class=PARAGRAPH>
                    "AI features use third-party services (such as Google Gemini). When you use AI identification, your submitted images and text may be processed by these third-party providers subject to their respective terms of service and privacy policies. We do not control how third-party AI providers process your data beyond what is necessary to deliver results to you."
                </p>

                // 7. Privacy and Data
                <h2 class=SECTION_HEADING>"7. Privacy and Data"</h2>
                <p class=PARAGRAPH>
                    "Your use of the Service is also governed by our "
                    <a href="/cookie-policy" class="font-medium underline transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">"Cookie Policy"</a>
                    ". We collect and process personal data as described therein."
                </p>
                <p class=PARAGRAPH>
                    "Your data is stored on our self-hosted infrastructure. We do not sell, rent, or share your personal information with third parties except as required to provide the Service (e.g., AI identification requests) or as required by law."
                </p>
                <p class=PARAGRAPH>
                    "You may delete your account and all associated data at any time through the "
                    <a href="/account/delete" class="font-medium underline transition-colors text-primary dark:text-primary-light dark:hover:text-accent-light hover:text-primary-light">"account deletion page"</a>
                    ". Upon deletion, all your data\u{2014}including plants, care history, photos, device credentials, and account settings\u{2014}is permanently removed from our systems."
                </p>

                // 8. Intellectual Property
                <h2 class=SECTION_HEADING>"8. Intellectual Property"</h2>
                <p class=PARAGRAPH>
                    "The Service and its original content (excluding User Content), features, and functionality are and will remain the exclusive property of Velamen and its licensors. The Service is protected by copyright, trademark, and other applicable laws. Our trademarks, trade names, and trade dress may not be used in connection with any product or service without prior written consent."
                </p>
                <p class=PARAGRAPH>
                    "You may not copy, modify, distribute, sell, or lease any part of the Service or its software, nor may you reverse-engineer or attempt to extract the source code of the software, unless applicable law expressly permits it."
                </p>

                // 9. Disclaimer of Warranties
                <h2 class=SECTION_HEADING>"9. Disclaimer of Warranties"</h2>
                <p class=PARAGRAPH>
                    "THE SERVICE IS PROVIDED \"AS IS\" AND \"AS AVAILABLE\" WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO IMPLIED WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, AND NON-INFRINGEMENT."
                </p>
                <p class=PARAGRAPH>
                    "We do not warrant that the Service will be uninterrupted, timely, secure, or error-free. We do not warrant the accuracy or reliability of any information obtained through the Service, including AI-generated content. Any reliance on such information is at your own risk."
                </p>

                // 10. Limitation of Liability
                <h2 class=SECTION_HEADING>"10. Limitation of Liability"</h2>
                <p class=PARAGRAPH>
                    "TO THE MAXIMUM EXTENT PERMITTED BY APPLICABLE LAW, IN NO EVENT SHALL VELAMEN, ITS OPERATORS, OR ITS LICENSORS BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, CONSEQUENTIAL, OR PUNITIVE DAMAGES, INCLUDING BUT NOT LIMITED TO LOSS OF DATA, LOSS OF PROFITS, OR LOSS OF GOODWILL, ARISING OUT OF OR IN CONNECTION WITH YOUR USE OF THE SERVICE."
                </p>
                <p class=PARAGRAPH>
                    "This includes, without limitation, damages arising from: the loss of plant data or care history; reliance on AI-generated identifications or care recommendations; unauthorized access to or alteration of your data; or any interruption or cessation of the Service."
                </p>

                // 11. Indemnification
                <h2 class=SECTION_HEADING>"11. Indemnification"</h2>
                <p class=PARAGRAPH>
                    "You agree to defend, indemnify, and hold harmless Velamen and its operators from and against any claims, damages, obligations, losses, liabilities, costs, or expenses (including reasonable attorney\u{2019}s fees) arising from: (a) your use of and access to the Service; (b) your violation of any term of these Terms; (c) your violation of any third-party right, including any intellectual property or privacy right; or (d) any User Content you upload to the Service."
                </p>

                // 12. Consequences for Violations
                <h2 class=SECTION_HEADING>"12. Consequences for Violations"</h2>
                <p class=PARAGRAPH>
                    "If you violate these Terms, we may take one or more of the following actions at our sole discretion and without prior notice:"
                </p>
                <ul class="mb-4 ml-6 space-y-1.5 text-sm leading-relaxed list-disc text-stone-600 dark:text-stone-300">
                    <li>"Issue a warning to your account"</li>
                    <li>"Temporarily suspend your access to the Service"</li>
                    <li>"Permanently terminate your account and delete all associated data"</li>
                    <li>"Remove or disable access to any content that violates these Terms"</li>
                    <li>"Take legal action if necessary"</li>
                </ul>

                // 13. Modifications to the Service
                <h2 class=SECTION_HEADING>"13. Modifications to the Service"</h2>
                <p class=PARAGRAPH>
                    "We reserve the right to modify, suspend, or discontinue the Service (or any part thereof) at any time, with or without notice. We shall not be liable to you or any third party for any modification, suspension, or discontinuation of the Service."
                </p>

                // 14. Changes to These Terms
                <h2 class=SECTION_HEADING>"14. Changes to These Terms"</h2>
                <p class=PARAGRAPH>
                    "We may revise these Terms from time to time. The most current version will always be available at this page. If a revision is material, we will make reasonable efforts to notify registered users. By continuing to access or use the Service after revisions become effective, you agree to be bound by the revised Terms."
                </p>

                // 15. Governing Law
                <h2 class=SECTION_HEADING>"15. Governing Law"</h2>
                <p class=PARAGRAPH>
                    "These Terms shall be governed by and construed in accordance with the laws of the United States, without regard to its conflict of law provisions. Any disputes arising under or in connection with these Terms shall be subject to the exclusive jurisdiction of the courts located within the United States."
                </p>

                // 16. Severability
                <h2 class=SECTION_HEADING>"16. Severability"</h2>
                <p class=PARAGRAPH>
                    "If any provision of these Terms is held to be unenforceable or invalid, that provision will be enforced to the maximum extent possible and the other provisions will remain in full force and effect."
                </p>

                // 17. Contact
                <h2 class=SECTION_HEADING>"17. Contact"</h2>
                <p class=PARAGRAPH>
                    "If you have any questions about these Terms, please contact us through the Velamen application."
                </p>
            </div>
        </main>
    }
}
