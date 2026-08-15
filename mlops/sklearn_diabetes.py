import mlflow

from sklearn.model_selection import train_test_split
from sklearn.datasets import load_diabetes
from sklearn.ensemble import RandomForestRegressor


# Enable MLflow's automatic experiment tracking for scikit-learn
mlflow.sklearn.autolog()

db = load_diabetes()
X_train, X_test, y_train, y_test = train_test_split(db.data, db.target, test_size=0.2)

model = RandomForestRegressor(n_estimators=100, max_depth=6, max_features=3)

# MLflow triggers logging automatically upon model fitting
model.fit(X_train, y_train)
