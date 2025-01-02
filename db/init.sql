--
-- PostgreSQL database dump
--

-- Dumped from database version 14.13
-- Dumped by pg_dump version 17.2 (Ubuntu 17.2-1.pgdg22.04+1)

ALTER TABLE public.todo ALTER COLUMN id DROP DEFAULT;
ALTER TABLE public.tag_message ALTER COLUMN id DROP DEFAULT;
ALTER TABLE public.tag ALTER COLUMN id DROP DEFAULT;
ALTER TABLE public."project-user" ALTER COLUMN id DROP DEFAULT;
ALTER TABLE public.project ALTER COLUMN id DROP DEFAULT;
ALTER TABLE public.model ALTER COLUMN id DROP DEFAULT;
DROP SEQUENCE public.todo_id_seq;
DROP TABLE public.todo;
DROP SEQUENCE public.tag_message_id_seq;
DROP TABLE public.tag_message;
DROP SEQUENCE public.tag_id_seq;
DROP TABLE public.tag;
DROP SEQUENCE public.project_id_seq;
DROP SEQUENCE public."project-user_id_seq";
DROP TABLE public."project-user";
DROP TABLE public.project;
DROP SEQUENCE public.model_version_seq;
DROP SEQUENCE public.model_id_seq;
DROP TABLE public.model;
DROP SCHEMA public;
--
-- TOC entry 4 (class 2615 OID 2200)
-- Name: public; Type: SCHEMA; Schema: -; Owner: -
--

CREATE SCHEMA public;


--
-- TOC entry 3423 (class 0 OID 0)
-- Dependencies: 4
-- Name: SCHEMA public; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON SCHEMA public IS 'standard public schema';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- TOC entry 210 (class 1259 OID 16386)
-- Name: model; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.model (
    version integer NOT NULL,
    id integer NOT NULL,
    modellink text,
    project_id bigint
);


--
-- TOC entry 209 (class 1259 OID 16385)
-- Name: model_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.model_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3424 (class 0 OID 0)
-- Dependencies: 209
-- Name: model_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.model_id_seq OWNED BY public.model.id;


--
-- TOC entry 211 (class 1259 OID 16392)
-- Name: model_version_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.model_version_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3425 (class 0 OID 0)
-- Dependencies: 211
-- Name: model_version_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.model_version_seq OWNED BY public.model.version;


--
-- TOC entry 213 (class 1259 OID 16394)
-- Name: project; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.project (
    id integer NOT NULL,
    name text NOT NULL,
    description text
);


--
-- TOC entry 215 (class 1259 OID 16401)
-- Name: project-user; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."project-user" (
    id integer NOT NULL,
    project_id bigint,
    user_id bigint
);


--
-- TOC entry 214 (class 1259 OID 16400)
-- Name: project-user_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public."project-user_id_seq"
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3426 (class 0 OID 0)
-- Dependencies: 214
-- Name: project-user_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public."project-user_id_seq" OWNED BY public."project-user".id;


--
-- TOC entry 212 (class 1259 OID 16393)
-- Name: project_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.project_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3427 (class 0 OID 0)
-- Dependencies: 212
-- Name: project_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.project_id_seq OWNED BY public.project.id;


--
-- TOC entry 217 (class 1259 OID 16406)
-- Name: tag; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag (
    id integer NOT NULL,
    title text NOT NULL,
    model_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    position_z real NOT NULL,
    position_x real NOT NULL,
    position_y real NOT NULL
);


--
-- TOC entry 216 (class 1259 OID 16405)
-- Name: tag_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tag_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3428 (class 0 OID 0)
-- Dependencies: 216
-- Name: tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tag_id_seq OWNED BY public.tag.id;


--
-- TOC entry 219 (class 1259 OID 16414)
-- Name: tag_message; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_message (
    id integer NOT NULL,
    text text,
    created_by bigint NOT NULL,
    tag_id bigint NOT NULL
);


--
-- TOC entry 218 (class 1259 OID 16413)
-- Name: tag_message_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.tag_message_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3429 (class 0 OID 0)
-- Dependencies: 218
-- Name: tag_message_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.tag_message_id_seq OWNED BY public.tag_message.id;


--
-- TOC entry 221 (class 1259 OID 16421)
-- Name: todo; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.todo (
    id integer NOT NULL,
    name character varying(250) NOT NULL
);


--
-- TOC entry 220 (class 1259 OID 16420)
-- Name: todo_id_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.todo_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3430 (class 0 OID 0)
-- Dependencies: 220
-- Name: todo_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.todo_id_seq OWNED BY public.todo.id;


--
-- TOC entry 3259 (class 2604 OID 16389)
-- Name: model id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.model ALTER COLUMN id SET DEFAULT nextval('public.model_id_seq'::regclass);


--
-- TOC entry 3260 (class 2604 OID 16397)
-- Name: project id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project ALTER COLUMN id SET DEFAULT nextval('public.project_id_seq'::regclass);


--
-- TOC entry 3261 (class 2604 OID 16404)
-- Name: project-user id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."project-user" ALTER COLUMN id SET DEFAULT nextval('public."project-user_id_seq"'::regclass);


--
-- TOC entry 3262 (class 2604 OID 16409)
-- Name: tag id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag ALTER COLUMN id SET DEFAULT nextval('public.tag_id_seq'::regclass);


--
-- TOC entry 3264 (class 2604 OID 16417)
-- Name: tag_message id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_message ALTER COLUMN id SET DEFAULT nextval('public.tag_message_id_seq'::regclass);


--
-- TOC entry 3265 (class 2604 OID 16424)
-- Name: todo id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.todo ALTER COLUMN id SET DEFAULT nextval('public.todo_id_seq'::regclass);


--
-- TOC entry 3406 (class 0 OID 16386)
-- Dependencies: 210
-- Data for Name: model; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- TOC entry 3409 (class 0 OID 16394)
-- Dependencies: 213
-- Data for Name: project; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- TOC entry 3411 (class 0 OID 16401)
-- Dependencies: 215
-- Data for Name: project-user; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- TOC entry 3413 (class 0 OID 16406)
-- Dependencies: 217
-- Data for Name: tag; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- TOC entry 3415 (class 0 OID 16414)
-- Dependencies: 219
-- Data for Name: tag_message; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- TOC entry 3417 (class 0 OID 16421)
-- Dependencies: 221
-- Data for Name: todo; Type: TABLE DATA; Schema: public; Owner: -
--



--
-- TOC entry 3431 (class 0 OID 0)
-- Dependencies: 209
-- Name: model_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.model_id_seq', 1, false);


--
-- TOC entry 3432 (class 0 OID 0)
-- Dependencies: 211
-- Name: model_version_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.model_version_seq', 1, false);


--
-- TOC entry 3433 (class 0 OID 0)
-- Dependencies: 214
-- Name: project-user_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public."project-user_id_seq"', 1, false);


--
-- TOC entry 3434 (class 0 OID 0)
-- Dependencies: 212
-- Name: project_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.project_id_seq', 1, false);


--
-- TOC entry 3435 (class 0 OID 0)
-- Dependencies: 216
-- Name: tag_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.tag_id_seq', 1, false);


--
-- TOC entry 3436 (class 0 OID 0)
-- Dependencies: 218
-- Name: tag_message_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.tag_message_id_seq', 1, false);


--
-- TOC entry 3437 (class 0 OID 0)
-- Dependencies: 220
-- Name: todo_id_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('public.todo_id_seq', 1, false);


-- Completed on 2025-01-02 23:28:06 CET

--
-- PostgreSQL database dump complete
--

