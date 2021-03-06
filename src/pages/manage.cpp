#include "Page.hpp"
#include "../MainFrame.hpp"
#include "../Manager.hpp"

class PageManageSelect : public Page {
protected:
    wxListBox* m_list;

    void onSelect(wxCommandEvent& e) {
        m_canContinue = m_list->GetSelection() != wxNOT_FOUND;
        m_frame->updateControls();
    }

public:
    PageManageSelect(MainFrame* frame) : Page(frame) {
        this->addText("Pick an installation to modify:");
        wxArrayString items;
        if (Manager::get()->isSuiteInstalled()) {
            items.push_back("Geode CLI");
        }
        for (auto& inst : Manager::get()->getInstallations()) {
            items.push_back(inst.m_path.wstring());
        }
        m_sizer->Add((m_list = new wxListBox(
            this, wxID_ANY, wxDefaultPosition, wxDefaultSize, items,
            wxLB_SINGLE | wxLB_HSCROLL
        )), 1, wxALL | wxEXPAND, 10);
        m_list->Bind(wxEVT_LISTBOX, &PageManageSelect::onSelect, this);
    }

    bool updateCLI() const {
        if (Manager::get()->isSuiteInstalled()) {
            return m_list->GetSelection() == 0;
        }
        return false;
    }

    Installation& which() const {
        // if suite is installed, dev is item #0
        // so we get item at index selected - 1 :)
        return Manager::get()->getInstallations().at(
            m_list->GetSelection() - Manager::get()->isSuiteInstalled()
        );
    }
};
REGISTER_PAGE(ManageSelect);

/////////////////

class PageManageCheck : public Page {
protected:
    wxStaticText* m_status;
    wxStaticText* m_nextInfo;
    VersionInfo m_newLoaderVersion;
    VersionInfo m_newCLIVersion;

    void enter() override {
        if (GET_EARLIER_PAGE(ManageSelect)->updateCLI()) {
            Manager::get()->checkCLIForUpdates(
                [this](std::string const& error) -> void {
                    wxMessageBox(
                        "Error checking for updates: " + error + 
                        ". Try again, and if the problem persists, contact "
                        "the Geode Development team for more help.",
                        "Error Updating",
                        wxICON_ERROR
                    );
                    this->setText(m_status, "Error: " + error);
                },
                [this](
                    VersionInfo const& current,
                    VersionInfo const& available
                ) -> void {
                    this->m_newCLIVersion = available;

                    this->setText(
                        m_status,
                        "Installed version: " + current.toString() + ",\n"
                        "Available version: " + available.toString()
                    );
                    if (current < available) {
                        this->setText(m_nextInfo, "Press \"Next\" to update Geode CLI.");
                        m_canContinue = true;
                        m_frame->updateControls();
                    } else {
                        this->setText(m_nextInfo, "You are up-to-date! :)");
                    }
                }
            );
        } else {
            Manager::get()->checkForUpdates(
                GET_EARLIER_PAGE(ManageSelect)->which(),
                [this](std::string const& error) -> void {
                    wxMessageBox(
                        "Error checking for updates: " + error + 
                        ". Try again, and if the problem persists, contact "
                        "the Geode Development team for more help.",
                        "Error Updating",
                        wxICON_ERROR
                    );
                    this->setText(m_status, "Error: " + error);
                },
                [this](
                    VersionInfo const& current,
                    VersionInfo const& available
                ) -> void {
                    this->m_newLoaderVersion = available;

                    this->setText(
                        m_status,
                        "Installed version: " + current.toString() + "\n"
                        "Available version: " + available.toString() + "\n"
                        "Branch: " +
                            (GET_EARLIER_PAGE(ManageSelect)->which().m_branch == DevBranch::Nightly ? 
                            "Nightly" : "Stable")
                    );
                    if (current < available) {
                        this->setText(m_nextInfo, "Press \"Next\" to update Geode.");
                    } else {
                        this->setText(m_nextInfo,
                            "You are up-to-date! :)\n"
                            "Press \"Next\" to reinstall Geode anyway, "
                            "or quit the installer if you don't want to "
                            "do that."
                        );
                    }
                    m_canContinue = true;
                    m_frame->updateControls();
                }
            );
        }
    }

public:
    PageManageCheck(MainFrame* frame) : Page(frame) {
        m_status = this->addText("Checking for updates...");
        m_nextInfo = this->addText("");
    }

    VersionInfo& getLoaderVersion() {
        return m_newLoaderVersion;
    }
    VersionInfo& getCLIVersion() {
        return m_newCLIVersion;
    }
};
REGISTER_PAGE(ManageCheck);

/////////////////

class PageManageOptBeta : public Page {
protected:
    wxCheckBox* m_check;

    void enter() override {
        m_skipThis = GET_EARLIER_PAGE(ManageSelect)->updateCLI();
    }

    void leave() override {
        m_skipThis = GET_EARLIER_PAGE(ManageSelect)->updateCLI();
    }

public:
    PageManageOptBeta(MainFrame* parent) : Page(parent) {
        if (!GET_EARLIER_PAGE(ManageSelect)->updateCLI()) {
            auto inst = GET_EARLIER_PAGE(ManageSelect)->which();
            if (inst.m_branch == DevBranch::Stable) {
                this->addText(
                    "Would you like to switch this installation "
                    "to use the beta version of the Geode loader? "
                    "Beta versions may be less stable and have "
                    "more bugs, but using beta versions lets "
                    "you try out new features ahead of time."
                );
                m_check = this->addToggle("Switch to the beta channel");
            } else {
                this->addText(
                    "Would you like to switch this installation "
                    "to use the stable version of the Geode loader? "
                    "You are currently using the beta channel, "
                    "which may be less stable and have more bugs, "
                    "but using beta versions lets you try out new "
                    "features ahead of time."
                );
                m_check = this->addToggle("Stay on beta channel");
                m_check->SetValue(true);
            }
        }
        m_canContinue = true;
    }
    
    DevBranch getBranch() const {
        return m_check->IsChecked() ? DevBranch::Nightly : DevBranch::Stable;
    }
};
REGISTER_PAGE(ManageOptBeta);

/////////////////

class PageManageUpdate : public Page {
protected:
    wxStaticText* m_status;
    wxGauge* m_gauge;

    void enter() override {
        if (GET_EARLIER_PAGE(ManageSelect)->updateCLI()) {
            Manager::get()->downloadCLI(
                [this](std::string const& str) -> void {
                    wxMessageBox(
                        "Error downloading the Geode CLI: " + str + 
                        ". Try again, and if the problem persists, contact "
                        "the Geode Development team for more help.",
                        "Error Updating",
                        wxICON_ERROR
                    );
                    this->setText(m_status, "Error: " + str);
                },
                [this](std::string const& text, int prog) -> void {
                    this->setText(m_status, "Downloading Geode CLI: " + text);
                    m_gauge->SetValue(prog);
                },
                [this](wxWebResponse const& wres) -> void {
                    auto installRes = Manager::get()->installCLI(
                        wres.GetDataFile().ToStdWstring()
                    );
                    if (!installRes) {
                        wxMessageBox(
                            "Error updating Geode CLI: " + installRes.error() + ". Try "
                            "again, and if the problem persists, contact "
                            "the Geode Development team for more help.",
                            "Error Updating",
                            wxICON_ERROR
                        );
                    } else {
                        Manager::get()->setCLIVersion(GET_EARLIER_PAGE(ManageCheck)->getCLIVersion());
                        m_frame->nextPage();
                    }
                }
            );
        } else {
            Manager::get()->installGeodeFor(
                GET_EARLIER_PAGE(ManageSelect)->which().m_path,
                GET_EARLIER_PAGE(ManageOptBeta)->getBranch(),
                [this](std::string const& str) -> void {
                    wxMessageBox(
                        "Error downloading the Geode loader: " + str + 
                        ". Try again, and if the problem persists, contact "
                        "the Geode Development team for more help.",
                        "Error Installing",
                        wxICON_ERROR
                    );
                    this->setText(m_status, "Error: " + str);
                },
                [this](std::string const& text, int prog) -> void {
                    this->setText(m_status, "Downloading Geode: " + text);
                    m_gauge->SetValue(prog / 2);
                },
                [this]() -> void {
                    GET_EARLIER_PAGE(ManageSelect)->which().m_loaderVersion = GET_EARLIER_PAGE(ManageCheck)->getLoaderVersion();
                    m_frame->nextPage();
                }
            );
        }
    }

public:
    PageManageUpdate(MainFrame* frame) : Page(frame) {
        if (GET_EARLIER_PAGE(ManageSelect)->updateCLI()) {
            this->addText("Updating Geode CLI");
        } else {
            this->addText("Updating installation");
        }

        m_status = this->addText("Connecting..."); 
        m_gauge = this->addProgressBar();

        m_canGoBack = false;
    }
};
REGISTER_PAGE(ManageUpdate);

/////////////////

class PageManageFinished : public Page {
protected:
    void enter() override {
        auto res = Manager::get()->saveData();
        if (!res) {
            wxMessageBox(
                "Unable to save installer data: " + res.error() + " - "
                "the installer will be unable to uninstall Geode!",
                "Error Saving",
                wxICON_ERROR
            );
        }
    }

public:
    PageManageFinished(MainFrame* frame) : Page(frame) {
        this->addText("Update complete!");
        m_canContinue = true;
        m_canGoBack = false;
    }
};
REGISTER_PAGE(ManageFinished);
