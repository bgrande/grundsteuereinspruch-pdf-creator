# 1.0.0
- split request handler functions into separate sub functions
- ausländische Nutzer?
- add a password (basic auth) (the customer's zip) to make it a bit more secure
-> needs a separate store (file) with the PLZ and creating this
- add request ids  https://github.com/imbolc/tower-request-id/blob/main/examples/logging.rs
- Use None or Error instead of empty string (mostly done)
- log success
- add possibility to give explanations to Einspruch reasons?
- add phone number in letter?
- collision check for file_id folder -> if already existing, try again (until no collision) -> preventing accidental override and data leaking
- delete mappings (after successfully used)
- Datenschutzvereinbarung doch zustimmen?
- really remove the pages from history!

# 0.3.0
- shorten (and/or group) the list of reasons 
- Payment/Checkoutseite mehr als solche (getrennt vom Rest) hervorheben
+ add Einspruch wg. Verfassungsklage wie in: https://youtu.be/nZDXlx8dWHA
+ fix folder + files creation on prod!
+ Einspruchtemplate tests
- Einspruchtemplate multiple pages -> print
- test cases:
    - 1. all possible fin plz+names -> should be 1 result each
    - 2. all possible PLZ (customer) to fin office -> Ausreißer
    - check if deadline date is correct
    - test different cases (multiple page, 10 senders, ...)
    + test at shuttle endpoint -> doesn't find files

# 0.2.0
+ split functions into separate files and modules
+ main endpoint for separate application
+ letter improvements
  + see https://www.t-online.de/finanzen/geld-vorsorge/steuern/id_100122008/fehler-im-grundsteuerbescheid-musterschreiben-fuer-ihren-einspruch.html)
  + improve wording in letter! Take both into account
+ Sonstiges sollte nicht in die Auflistung!
+ use logging instead of tracing (with error!, debug!, ...)
  -> https://docs.rs/env_logger/latest/env_logger/
+ use error page template

# 0.1.0
+ download pdf
+ how to get the tax office db on deploy?
  + einchecken in git?
+ templates in deploy: einchecken?
+ email, etc.: secrets + einchecken? 
+ how to integrate the templates (separate repo) -
  - either git submodule or wget/curl before deploy? (in deploy script)
  - include at compile vs deliver with deployment
+ Formulierungen + snippets für Widerspruch/Einspruch schreiben
    + Formulierung im Schreiben für Proforma Widerspruch/Einspruch: ???
      -> Begründung wird nachgereicht
+ make sure all required data is available + fix missing
+ deploy to shuttle.rs
  + fix with secrets
  + add config to vcs
  + add db to vcs
+ create formresult page hashed email (or probably even better the result id of the form sent)
    + separate page/endpoint for thank you page (redirect after form fill)
        - when creating the files also create a mapping hash(email) to file_id
        - the mapping should only be called once and then get deleted
        - the separate page should call the endpoint (poll until valid result) and get the actual URL for the overview and redirect to that
+ format Bescheid vom Datum(s)
+ add date for deadline
+ fix index template rendering
+ Rust backend für PDF Erstellung
    + tera für template2html?
    + api with actix
    + queries für pdf,html,get
    + log meta info 
    + log errors
    + write payload into json
    + create html pages
      + create letter
      + create invoice
      + create index
      + create list/tips
    + create PDF files
    + prevent second empty page in PDFs
    + pages index, formresult
    + sql queries + tax office db integration
      + use the tax office data
      + how to handle multiple finance office results for same zip? like DUS 40476 