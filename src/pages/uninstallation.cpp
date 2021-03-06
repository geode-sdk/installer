#include "Page.hpp"
#include "../MainFrame.hpp"
#include "../Manager.hpp"
#include <wx/dataview.h>

class PageUninstallStart : public Page {
protected:
    bool m_uninstallEverything = true;

    void onSelect(wxCommandEvent& e) override {
        switch (e.GetId()) {
            case 0: {
                if (
                    Manager::get()->isSuiteInstalled() &&
                    Manager::get()->needRequestAdminPriviledges()
                ) {
                    wxMessageBox(
                        "You need to run the installer as "
                        "administrator in order to uninstall "
                        "the developer tools.",
                        "Admin Priviledges Required",
                        wxICON_ERROR
                    );
                    m_uninstallEverything = false;
                    m_canContinue = false;
                    m_frame->updateControls();
                    return;
                }
                m_uninstallEverything = true;
                
            } break;
            case 1: m_uninstallEverything = false; break;
            default: break;
        }
        m_canContinue = true;
        m_frame->updateControls();
    }

public:
    PageUninstallStart(MainFrame* frame) : Page(frame) {
        this->addText("What would you like to uninstall?");
        this->addSelect({
            "Uninstall everything",
            "Choose which parts to uninstall"
        });
        m_canContinue = true;
        if (
            Manager::get()->isSuiteInstalled() &&
            Manager::get()->needRequestAdminPriviledges()
        ) {
            wxMessageBox(
                "You need to run the installer as "
                "administrator in order to uninstall "
                "the developer tools.",
                "Admin Priviledges Required",
                wxICON_ERROR
            );
            m_uninstallEverything = false;
            m_canContinue = false;
        }
    }

    bool completeUninstall() const {
        return m_uninstallEverything;
    }
};
REGISTER_PAGE(UninstallStart);

/////////////////

class PageUninstallSelect : public Page {
protected:
    wxCheckBox* m_devCheck = nullptr;
    wxStaticText* m_devInfo = nullptr;
    wxDataViewListCtrl* m_list;
    std::unordered_map<size_t, Installation> m_items;
    std::set<size_t> m_selected;

    void enter() override {
        if (
            Manager::get()->isSuiteInstalled() &&
            Manager::get()->needRequestAdminPriviledges()
        ) {
            m_devCheck->Disable();
            m_devCheck->SetValue(false);
            m_devInfo->Show();
        } else {
            if (m_devCheck) m_devCheck->Enable();
            if (m_devCheck) m_devInfo->Hide();
        }
        m_skipThis = GET_EARLIER_PAGE(UninstallStart)->completeUninstall();
    }

    void onSelectPart(wxDataViewEvent& e) {
        auto row = m_list->GetSelectedRow();
        if (m_list->GetToggleValue(row, 1)) {
            m_selected.insert(row);
        } else {
            m_selected.erase(row);
        }
    }

public:
    PageUninstallSelect(MainFrame* frame) : Page(frame) {
        this->addText("Please select which parts of Geode to uninstall.");

        if (Manager::get()->isSuiteInstalled()) {
            m_devCheck = this->addToggle<PageUninstallSelect>("Uninstall Developer SDK", nullptr);
            m_devInfo = this->addText(
                "You need to run the installer as "
                "administrator to uninstall the "
                "developer tools."
            );
            m_devInfo->Hide();
        }

        m_list = new wxDataViewListCtrl(this, wxID_ANY);

        m_list->Bind(wxEVT_DATAVIEW_ITEM_VALUE_CHANGED, &PageUninstallSelect::onSelectPart, this);
        m_list->AppendTextColumn("Location", wxDATAVIEW_CELL_INERT, m_frame->GetSize().x - 150);
        m_list->AppendToggleColumn("Uninstall");

        size_t ix = 0;
        for (auto& i : Manager::get()->getInstallations()) {
            wxVector<wxVariant> data;
            data.push_back(wxVariant(i.m_path.wstring()));
            data.push_back(wxVariant(false));
            m_list->AppendItem(data);
            m_items.insert({ ix, i });
            ix++;
        }
        m_sizer->Add(m_list, 1, wxALL | wxEXPAND, 10);
        
        m_canContinue = true;
    }

    bool shouldUninstallSuite() const {
        if (GET_EARLIER_PAGE(UninstallStart)->completeUninstall()) return true;
        return m_devCheck && m_devCheck->IsChecked();
    }

    bool shouldUninstall(Installation const& inst) const {
        if (GET_EARLIER_PAGE(UninstallStart)->completeUninstall()) return true;
        for (auto& sel : m_selected) {
            if (m_items.count(sel) && m_items.at(sel) == inst) {
                return true;
            }
        }
        return false;
    }

    size_t selectedAmount() const {
        if (GET_EARLIER_PAGE(UninstallStart)->completeUninstall()) {
            return m_items.size();
        }
        return m_selected.size();
    }
};
REGISTER_PAGE(UninstallSelect);

/////////////////

class PageUninstallDeleteData : public Page {
protected:
    wxCheckBox* m_box;
    wxStaticText* m_info;
    bool m_deleteData = true;

    void leave() override {
        m_deleteData = m_box->IsChecked();
    }

    void enter() override {
        if (!GET_EARLIER_PAGE(UninstallSelect)->selectedAmount()) {
            m_skipThis = true;
        }
        this->setText(m_info, 
            "Do you also want to delete all save data associated "
            "with " + std::string(
                GET_EARLIER_PAGE(UninstallStart)->completeUninstall() ?
                    "Geode" : "the selected parts"
            ) + "? This means that all Geode- and mod-related settings, "
            "save data, etc. will be lost. This will not affect your "
            "normal Geometry Dash save data."
        );
    }

public:
    PageUninstallDeleteData(MainFrame* parent) : Page(parent) {
        m_info = this->addText("");
        this->addText(
            "Note that for some GDPSes this installer may be "
            "unable to accurately detect where Geode's save data "
            "is located. In these cases, you will have to manually "
            "remove the save data. Contact the GDPS's owner for "
            "information on where its save data lies."
        );

        m_box = this->addToggle<PageUninstallDeleteData>("Delete save data", nullptr);
        m_box->SetValue(m_deleteData);

        m_canContinue = true;
    }

    bool shouldDeleteData() const {
        return m_deleteData;
    }
};
REGISTER_PAGE(UninstallDeleteData);

/////////////////

class PageUninstall : public Page {
protected:
    wxGauge* m_gauge;

    void enter() override {
        for (auto& inst : Manager::get()->getInstallations()) {
            if (GET_EARLIER_PAGE(UninstallSelect)->shouldUninstall(inst)) {
                auto ur = Manager::get()->uninstallGeodeFrom(inst);
                if (!ur) {
                    wxMessageBox(
                        "Unable to uninstall Geode from " + inst.m_path.string() + ": " +
                        ur.error() + ". You may need to manually remove the files; "
                        "contact the Geode Development Team for more information.",
                        "Error Uninstalling",
                        wxICON_ERROR
                    );
                }
                if (GET_EARLIER_PAGE(UninstallDeleteData)->shouldDeleteData()) {
                    auto dr = Manager::get()->deleteSaveDataFrom(inst);
                    if (!dr) {
                        // don't ask me why. ur.error() sometimes throws bad alloc.
                        // this is fantastic code i think
                        try {
                            wxMessageBox(
                                "Unable to delete Geode save data from " + inst.m_path.string() + ": " +
                                ur.error() + ". You may need to manually remove "
                                "the files; if the given installation is a GDPS, "
                                "contact its owner for help. Otherwise, contact "
                                "the Geode Development Team for more information.",
                                "Error Uninstalling",
                                wxICON_ERROR
                            );
                        } catch(std::exception&) {
                            wxMessageBox(
                                "Unable to delete Geode save data from " + inst.m_path.string() + 
                                ". You may need to manually remove the files; if "
                                "the given installation is a GDPS, contact its "
                                "owner for help. Otherwise, contact the Geode "
                                "Development Team for more information.",
                                "Error Uninstalling",
                                wxICON_ERROR
                            );
                        }
                    }
                }
            }
        }
        m_gauge->SetValue(33);
        m_frame->Update();
        if (GET_EARLIER_PAGE(UninstallSelect)->shouldUninstallSuite()) {
            auto sr = Manager::get()->uninstallSuite();
            if (!sr) {
                wxMessageBox(
                    "Unable to uninstall the Geode SDK: " + sr.error() +
                    ". Contact the Geode Development Team for more "
                    "information.",
                    "Error Uninstalling",
                    wxICON_ERROR
                );
            }
        }
        m_gauge->SetValue(66);
        m_frame->Update();
        if (GET_EARLIER_PAGE(UninstallStart)->completeUninstall()) {
            auto dr = Manager::get()->deleteData();
            if (!dr) {
                wxMessageBox(
                    "Unable to delete installer data: " + dr.error() +
                    ". Contact the Geode Development Team for more "
                    "information.",
                    "Error Uninstalling",
                    wxICON_ERROR
                );
            }
        }
        m_gauge->SetValue(100);
        m_frame->Update();
        m_canContinue = true;
        m_skipThis = true;
    }

public:
    PageUninstall(MainFrame* parent) : Page(parent) {
        this->addText("Uninstalling...");
        m_gauge = this->addProgressBar();
        m_canContinue = true;
        m_canGoBack = true;
    }
};
REGISTER_PAGE(Uninstall);

/////////////////

class PageUninstallFinished : public Page {
protected:
    void enter() override {
        if (!GET_EARLIER_PAGE(UninstallStart)->completeUninstall()) {
            auto res = Manager::get()->saveData();
            if (!res) {
                wxMessageBox(
                    "Unable to save installer data: " + res.error(),
                    "Error",
                    wxICON_ERROR
                );
            }
        }
    }

public:
    PageUninstallFinished(MainFrame* frame) : Page(frame) {
        this->addText(
            "Uninstalling finished! "
            "Geode should now be removed from " + std::string(
                GET_EARLIER_PAGE(UninstallStart)->completeUninstall() ?
                    "your computer" : "the selected parts"
            ) + " :)"
        );
        this->addText(
            "If you find parts of Geode still lurking "
            "around, contact the Geode Development Team "
            "for help."
        );
        this->addButton("Support Discord Server", &MainFrame::onDiscord, m_frame);

        m_canContinue = true;
        m_canGoBack = false;
    }
};
REGISTER_PAGE(UninstallFinished);

