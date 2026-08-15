import psycopg2

from contextlib import (
    asynccontextmanager,
    closing,
)
from fastapi import FastAPI

@asynccontextmanager
async def psql_lifespan(app: FastAPI) -> None:
    """ """

    app.db = psycopg2.connect(
        database="postgres",
        host="localhost",
        port="5432",
        user="postgres",
        password="",
    )

    yield

    app.db.close()

app = FastAPI(lifespan=psql_lifespan)


@app.get("/db/customer")
async def get_db_customer():
    """"""

    with closing(app.db.cursor()) as cur:
        cur.execute("SELECT * FROM customer")
        result = cur.fetchall()

    return result


@app.get("/db/product")
async def get_db_product():
    """"""

    with closing(app.db.cursor()) as cur:
        cur.execute("SELECT * FROM product")
        result = cur.fetchall()

    return result



@app.get("/db/order")
async def get_db_order():
    """"""

    with closing(app.db.cursor()) as cur:
        cur.execute("SELECT * FROM order")
        result = cur.fetchall()

    return result


@app.get("/db/order_detail")
async def get_db_order_detail():
    """"""

    with closing(app.db.cursor()) as cur:
        cur.execute("SELECT * FROM order_detail")
        result = cur.fetchall()

    return result

