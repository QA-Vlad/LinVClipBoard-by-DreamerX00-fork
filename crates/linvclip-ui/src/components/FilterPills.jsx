import { useTranslation } from "../i18n/index.jsx";

const FILTERS = [
    { id: "all",    labelKey: "filters.all" },
    { id: "text",   labelKey: "filters.text" },
    { id: "image",  labelKey: "filters.images" },
    { id: "files",  labelKey: "filters.files" },
    { id: "pinned", labelKey: "filters.pinned" },
];

function FilterPills({ activeFilter, onFilterChange }) {
    const { t } = useTranslation();

    return (
        <div className="filter-pills" role="radiogroup" aria-label={t("filters.all")}>
            {FILTERS.map((f) => (
                <button
                    key={f.id}
                    role="radio"
                    aria-checked={activeFilter === f.id}
                    className={`pill${activeFilter === f.id ? " pill-active" : ""}`}
                    onClick={() => onFilterChange(f.id)}
                >
                    {t(f.labelKey)}
                </button>
            ))}
        </div>
    );
}

export default FilterPills;
