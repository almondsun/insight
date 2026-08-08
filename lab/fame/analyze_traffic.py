#!/usr/bin/env python3
"""Preregistered-shape classifier harness for synthetic Fame traces only."""
import argparse
import json
from pathlib import Path

import pandas as pd
from sklearn.ensemble import HistGradientBoostingClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.metrics import balanced_accuracy_score, roc_auc_score
from sklearn.model_selection import StratifiedKFold, cross_val_predict
from sklearn.pipeline import make_pipeline
from sklearn.preprocessing import StandardScaler


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("trace", type=Path)
    args = parser.parse_args()
    data = pd.read_csv(args.trace)
    required = {"label", "packet_count", "byte_count", "duration_ms", "interarrival_mean_ms"}
    if not required.issubset(data.columns):
        raise SystemExit(f"missing columns: {sorted(required - set(data.columns))}")
    if set(data["label"]) != {"dummy", "real"}:
        raise SystemExit("trace labels must be exactly dummy and real")
    y = (data.pop("label") == "real").astype(int)
    x = data[["packet_count", "byte_count", "duration_ms", "interarrival_mean_ms"]]
    cv = StratifiedKFold(n_splits=5, shuffle=True, random_state=20260808)
    models = {
        "logistic": make_pipeline(StandardScaler(), LogisticRegression(random_state=20260808)),
        "gradient": HistGradientBoostingClassifier(random_state=20260808),
    }
    results = {}
    for name, model in models.items():
        probability = cross_val_predict(model, x, y, cv=cv, method="predict_proba")[:, 1]
        prediction = probability >= 0.5
        results[name] = {"rocAuc": roc_auc_score(y, probability), "balancedAccuracy": balanced_accuracy_score(y, prediction)}
    print(json.dumps({"syntheticOnly": True, "productionPrivacyClaim": False, "diagnostics": results}, indent=2))


if __name__ == "__main__":
    main()
