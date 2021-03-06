#include "Page.hpp"
#include "../MainFrame.hpp"

std::unordered_map<PageID, PageGen> g_generators;
std::unordered_map<PageID, Page*> g_generated;

Page::Page(MainFrame* parent) : wxPanel(parent) {
    m_frame = parent;
    m_sizer = new wxBoxSizer(wxVERTICAL);
    this->SetSizer(m_sizer);
    this->Hide();
}

void Page::enter() {}
void Page::leave() {}
void Page::resize() {
    for (auto& [label, str] : m_labels) {
        label->SetLabel(str);
        label->Wrap(this->GetSize().x - 20);
    }
}
void Page::onSelect(wxCommandEvent&) {}

void Page::setText(wxStaticText* text, wxString const& newText) {
    m_labels[text] = newText;
    m_frame->Layout();
    this->resize();
    this->Update();
    m_frame->Layout();
}

wxStaticText* Page::addText(wxString const& text) {
    auto label = new wxStaticText(this, wxID_ANY, text);
    m_sizer->Add(label, 0, wxALL | wxEXPAND, 10);
    m_labels.insert({ label, text });
    return label;
}

wxTextCtrl* Page::addLongText(wxString const& text) {
    auto ctrl = new wxTextCtrl(
        this, wxID_ANY, text,
        wxDefaultPosition, wxDefaultSize,
        wxTE_MULTILINE | wxTE_READONLY
    );
    m_sizer->Add(ctrl, 1, wxALL | wxEXPAND, 10);
    return ctrl;
}

wxCheckBox* Page::addToggle(wxString const& text) {
    auto btn = new wxCheckBox(this, wxID_ANY, text);
    m_sizer->Add(btn, 0, wxALL, 10);
    return btn;
}

wxGauge* Page::addProgressBar() {
    auto bar = new wxGauge(this, wxID_ANY, 100);
    m_sizer->Add(bar, 0, wxALL | wxEXPAND, 10);
    return bar;
}

void Page::addSelect(std::initializer_list<wxString> const& select) {
    size_t ix = 0;
    for (auto& s : select) {
        auto gdps = new wxRadioButton(
            this, ix,
            s, wxDefaultPosition, wxDefaultSize, (!ix ? wxRB_GROUP : 0)
        );
        gdps->Bind(wxEVT_RADIOBUTTON, &Page::onSelect, this);
        m_sizer->Add(gdps, 0, wxALL, 10);
        ix++;
    }
}

void Page::addSelectWithIDs(std::initializer_list<std::pair<wxString, size_t>> const& select) {
    size_t ix = 0;
    for (auto& [s, id] : select) {
        auto gdps = new wxRadioButton(
            this, id,
            s, wxDefaultPosition, wxDefaultSize, (!ix ? wxRB_GROUP : 0)
        );
        gdps->Bind(wxEVT_RADIOBUTTON, &Page::onSelect, this);
        m_sizer->Add(gdps, 0, wxALL, 10);
        ix++;
    }
}

Page* Page::getPage(PageID id, MainFrame* frame) {
    if (g_generated.count(id)) {
        return g_generated.at(id);
    }
    if (g_generators.count(id)) {
        auto p = g_generators.at(id)(frame);
        g_generated.insert({ id, p });
        return p;
    }
    return nullptr;
}

void Page::registerPage(PageID id, PageGen gen) {
    g_generators.insert({ id, gen });
}
