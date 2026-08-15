"""
Traing a CNN on the MNIST Fashion dataset with MLflow experiment tracking.

File created: 2025-03-06
Last updated: 2025-03-06
"""

import argparse
import mlflow
import numpy as np
import torch
import torch.nn as nn

from mlflow.models.signature import (
    infer_signature,
    ModelSignature,
)
from torch.utils.data import (
    DataLoader,
    Dataset,
)
from torchinfo import summary
from torchmetrics import (
    Accuracy,
    Metric,
)
from torchvision import datasets
from torchvision.transforms import ToTensor


class CoolNet(nn.Module):
    def __init__(self, n_classes: int, dropout_p: float):
        super(CoolNet, self).__init__()

        self._n_classes = n_classes
        self._dropout_p = dropout_p

        self._conv1 = nn.Sequential(
            nn.Conv2d(in_channels=1, out_channels=16, kernel_size=(3, 3), padding=1),
            nn.BatchNorm2d(16),
            nn.ReLU(),
            nn.MaxPool2d(kernel_size=(2, 2), stride=(2, 2)),
        )

        self._conv2 = nn.Sequential(
            nn.Conv2d(in_channels=16, out_channels=32, kernel_size=(3, 3), padding=0),
            nn.BatchNorm2d(32),
            nn.ReLU(),
            nn.MaxPool2d(kernel_size=(2, 2)),
        )

        # self._fc1 = nn.Linear(in_features=32 * 6 * 6, out_features=600)
        self._fc1 = nn.LazyLinear(out_features=512)
        self._fc2 = nn.LazyLinear(out_features=128)
        self._fc3 = nn.LazyLinear(out_features=n_classes)
        self._dropout = nn.Dropout(p=dropout_p)

    def forward(self, x: torch.tensor) -> torch.tensor:
        h = self._conv1(x)
        h = self._conv2(h)

        # This flattens the tensor
        h = h.view(h.size(0), -1)

        h = self._fc1(h)
        h = self._dropout(h)
        h = self._fc2(h)
        h = self._fc3(h)

        return h


def train(
    dataloader: DataLoader,
    model: nn.Module,
    device: str,
    loss_fn: nn.Module,
    metrics_fn: Metric,
    optimizer: torch.optim.Optimizer,
    epoch: int,
):
    """Train the neural net on a single pass of the dataloader."""

    model.train()
    for batch, (X, y) in enumerate(dataloader):
        X, y = X.to(device), y.to(device)

        pred = model(X)
        loss = loss_fn(pred, y)
        acc = metrics_fn(pred, y)

        optimizer.zero_grad()
        loss.backward()
        optimizer.step()

        if batch % 100 == 0:
            loss, curr = loss.item(), batch
            step = batch // 100 * (epoch + 1)
            
            mlflow.log_metric("loss", f"{loss:.3f}", step=step)
            mlflow.log_metric("accuracy", f"{acc:.3f}", step=step)

            print(f"Training: loss={loss:.3f}  acc={acc:.3f}  [{curr} / {len(dataloader)}]")


def evaluate(
    dataloader: DataLoader,
    model: nn.Module,
    device: str,
    loss_fn: nn.Module,
    metrics_fn: Metric,
    epoch: int,
):
    """Evaluate the neural net on a single pass of the dataloader."""

    n_batches = len(dataloader)
    model.eval()
    eval_loss, eval_acc = 0.0, 0.0

    with torch.no_grad():
        for X, y in dataloader:
            X, y = X.to(device), y.to(device)
            pred = model(X)
            eval_loss += loss_fn(pred, y).item()
            eval_acc += metrics_fn(pred, y)

    eval_loss /= n_batches
    eval_acc /= n_batches

    mlflow.log_metric("eval_loss", f"{eval_loss:.3f}", step=epoch)
    mlflow.log_metric("eval_acc", f"{eval_acc:.3f}", step=epoch)

    print(f"Evaluation: loss={eval_loss:.3f}  acc={eval_acc:.3f}")


if __name__ == "__main__":

    parser: argparse.ArgumentParser = argparse.ArgumentParser()
    parser.add_argument(
        "--epochs", default=10, type=int, help="The number of epochs to train for"
    )
    parser.add_argument(
        "--batch-size", default=64, type=int,
        help="The batch size to use for training and evaluation",
    )
    parser.add_argument(
        "--lr", default=3e-4, type=float,
        help="The learning rate to use for training",
    )
    parser.add_argument(
        "--dropout-p", default=0.2, type=float,
        help="The dropout layer probability to use for training",
    )
    parser.add_argument(
        "--run", type=str, help="The run identifier for the MLflow experiment",
    )

    args: argparse.Namespace = parser.parse_args()

    device: str = "cuda" if torch.cuda.is_available() else "cpu"
    print(f"Running on device: {device}")

    train_data: Dataset = datasets.FashionMNIST(
        root="data",
        train=True,
        download=True,
        transform=ToTensor(),
    )

    test_data: Dataset = datasets.FashionMNIST(
        root="data",
        train=False,
        download=True,
        transform=ToTensor(),
    )


    print(f"MNIST image size: {train_data[0][0].shape}")
    print(f"Size of training dataset: {len(train_data)}")
    print(f"Size of testing dataset: {len(test_data)}")

    batch_size: int = args.batch_size

    train_loader: DataLoader = DataLoader(train_data, shuffle=True, batch_size=batch_size)
    test_loader: DataLoader = DataLoader(test_data, shuffle=False, batch_size=batch_size)

    experiment_run: str = args.run

    mlflow.set_tracking_uri("http://localhost:5000")
    mlflow.set_experiment("pytorch-mnist-coolnet")

    epochs: int = args.epochs
    lr: float = args.lr
    dropout_p: float = args.dropout_p

    model: nn.Module = CoolNet(n_classes=10, dropout_p=dropout_p).to(device)
    loss_fn: nn.Module = nn.CrossEntropyLoss()
    metrics_fn: Metric = Accuracy(task="multiclass", num_classes=10).to(device)
    optimizer: torch.optim.Optimizer = torch.optim.SGD(model.parameters(), lr=lr)

    with mlflow.start_run(run_name=args.run):
        params = {
            "epochs": epochs,
            "learning_rate": lr,
            "batch_size": batch_size,
            "loss_fn": loss_fn.__class__.__name__,
            "metrics_fn": metrics_fn.__class__.__name__,
            "optimizer": optimizer.__class__.__name__,
        }

        mlflow.log_params(params)

        with open("model_summary.txt", "w") as f:
            f.write(str(summary(model)))

        mlflow.log_artifact("model_summary.txt")

        # Evaluate the model before training to get "baseline"
        evaluate(test_loader, model, device, loss_fn, metrics_fn, epoch=0)

        for epoch in range(1, epochs + 1):
            print(f" =========== Epoch {epoch} ===========")

            train(train_loader, model, device, loss_fn, metrics_fn, optimizer, epoch=epoch)
            evaluate(test_loader, model, device, loss_fn, metrics_fn, epoch=epoch)

        with torch.no_grad():
            input_example: np.ndarray = train_data[0][0].unsqueeze(0).cpu().numpy()
            model_signature: ModelSignature = infer_signature(
                input_example,
                model(train_data[0][0].unsqueeze(0)).cpu().numpy(),
            )

            mlflow.pytorch.log_model(model, "coolnet", signature=model_signature, input_example=input_example)

