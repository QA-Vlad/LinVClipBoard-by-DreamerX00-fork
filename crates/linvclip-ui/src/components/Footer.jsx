import { useTranslation } from "../i18n/index.jsx";

function Footer() {
    const { t } = useTranslation();

    return (
        <footer className="app-footer" role="contentinfo">
            <span className="footer-text">{t("app.footer")}</span>
        </footer>
    );
}

export default Footer;
