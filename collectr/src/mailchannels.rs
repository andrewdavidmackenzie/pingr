use serde::{Serialize};

#[derive(Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    type_: String,
    value: String,
}

impl Content {
    pub fn new(type_: impl Into<String>, value: impl Into<String>) -> Content {
        Content {
            type_: type_.into(),
            value: value.into()
        }
    }
}

#[derive(Serialize)]
pub struct Contact {
    email: String,
    name: String,
}

impl Contact {
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Contact {
        Contact {
            email: email.into(),
            name: name.into()
        }
    }
}

#[derive(Serialize)]
pub struct Dkim {
    #[serde(rename = "dkim_domain")]
    domain: String,
    #[serde(rename = "dkim_selector")]
    selector: String,
    #[serde(rename = "dkim_private_key")]
    private_key: String,
}

impl Dkim {
    pub fn new(
        domain: impl Into<String>,
        selector: impl Into<String>,
        private_key: impl Into<String>
    ) -> Dkim {
        Dkim {
            domain: domain.into(),
            selector: selector.into(),
            private_key: private_key.into(),
        }
    }
}

#[derive(Serialize)]
pub struct Personalization {
    to: Vec<Contact>,
    #[serde(flatten)]
    dkim: Dkim,
}

impl Personalization {
    pub fn new(dkim: Dkim) -> Personalization {
        Personalization {
            to: Default::default(),
            dkim
        }
    }

    pub fn to(mut self, contact: Contact) -> Self {
        self.to.push(contact);
        self
    }

}

#[derive(Serialize)]
pub struct Body {
    personalizations: Vec<Personalization>,
    from: Contact,
    subject: String,
    content: Vec<Content>
}

impl Body {
    pub fn new(from: Contact, subject: impl Into<String>) -> Body {
        Body {
            personalizations: Default::default(),
            from,
            subject: subject.into(),
            content: Default::default(),
        }
    }

    pub fn personalization(mut self, personalization: Personalization) -> Self {
        self.personalizations.push(personalization);
        self
    }

    pub fn content(mut self, content: Content) -> Self {
        self.content.push(content);
        self
    }
}