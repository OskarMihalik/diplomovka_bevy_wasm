--
-- PostgreSQL database dump
--

-- Dumped from database version 14.13
-- Dumped by pg_dump version 17.2 (Ubuntu 17.2-1.pgdg22.04+1)

--
-- TOC entry 209 (class 1259 OID 16385)
-- Name: model; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.model (
    version integer NOT NULL,
    id serial NOT NULL,
    modellink text,
    project_id bigint
);


--
-- TOC entry 210 (class 1259 OID 16390)
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
-- TOC entry 3423 (class 0 OID 0)
-- Dependencies: 210
-- Name: model_version_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.model_version_seq OWNED BY public.model.version;


--
-- TOC entry 211 (class 1259 OID 16391)
-- Name: project; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.project (
    id serial NOT NULL,
    name text NOT NULL,
    description text
);


--
-- TOC entry 212 (class 1259 OID 16396)
-- Name: project-user; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."project-user" (
    id serial NOT NULL,
    project_id bigint,
    user_id bigint
);


--
-- TOC entry 213 (class 1259 OID 16399)
-- Name: tag; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag (
    id serial NOT NULL,
    title text NOT NULL,
    model_id bigint NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    position_z real NOT NULL,
    position_x real NOT NULL,
    position_y real NOT NULL
);


--
-- TOC entry 214 (class 1259 OID 16405)
-- Name: tag_message; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.tag_message (
    id serial NOT NULL,
    text text,
    created_by bigint NOT NULL,
    tag_id bigint NOT NULL
);


--
-- TOC entry 215 (class 1259 OID 16410)
-- Name: todo; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public.todo (
    id serial NOT NULL,
    name character varying(250) NOT NULL
);


--
-- TOC entry 217 (class 1259 OID 16414)
-- Name: todo_id_seq1; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE public.todo_id_seq1
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- TOC entry 3424 (class 0 OID 0)
-- Dependencies: 217
-- Name: todo_id_seq1; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE public.todo_id_seq1 OWNED BY public.todo.id;


--
-- TOC entry 218 (class 1259 OID 16415)
-- Name: user; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE public."user" (
    name character varying NOT NULL,
    email character varying NOT NULL,
    avatar_link character varying,
    id serial NOT NULL
);


--
-- TOC entry 3260 (class 2604 OID 16421)
-- Name: todo id; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.todo ALTER COLUMN id SET DEFAULT nextval('public.todo_id_seq1'::regclass);


--
-- TOC entry 3262 (class 2606 OID 16423)
-- Name: model model_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.model
    ADD CONSTRAINT model_pk PRIMARY KEY (id);


--
-- TOC entry 3264 (class 2606 OID 16425)
-- Name: project project_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.project
    ADD CONSTRAINT project_pk PRIMARY KEY (id);


--
-- TOC entry 3266 (class 2606 OID 16427)
-- Name: project-user projectuser_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."project-user"
    ADD CONSTRAINT projectuser_pk PRIMARY KEY (id);


--
-- TOC entry 3268 (class 2606 OID 16429)
-- Name: tag tag_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_pk PRIMARY KEY (id);


--
-- TOC entry 3270 (class 2606 OID 16431)
-- Name: tag_message tagmessage_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_message
    ADD CONSTRAINT tagmessage_pk PRIMARY KEY (id);


--
-- TOC entry 3272 (class 2606 OID 16433)
-- Name: user user_pk; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT user_pk PRIMARY KEY (id);


--
-- TOC entry 3273 (class 2606 OID 16434)
-- Name: model model_project_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.model
    ADD CONSTRAINT model_project_fk FOREIGN KEY (project_id) REFERENCES public.project(id);


--
-- TOC entry 3274 (class 2606 OID 16439)
-- Name: project-user project_user_project_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."project-user"
    ADD CONSTRAINT project_user_project_fk FOREIGN KEY (project_id) REFERENCES public.project(id);


--
-- TOC entry 3275 (class 2606 OID 16444)
-- Name: project-user project_user_user_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public."project-user"
    ADD CONSTRAINT project_user_user_fk FOREIGN KEY (user_id) REFERENCES public."user"(id);


--
-- TOC entry 3276 (class 2606 OID 16449)
-- Name: tag tag_model_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_model_fk FOREIGN KEY (model_id) REFERENCES public.model(id);


--
-- TOC entry 3277 (class 2606 OID 16454)
-- Name: tag_message tagmessage_tag_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_message
    ADD CONSTRAINT tagmessage_tag_fk FOREIGN KEY (tag_id) REFERENCES public.tag(id);


--
-- TOC entry 3278 (class 2606 OID 16459)
-- Name: tag_message tagmessage_user_fk; Type: FK CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY public.tag_message
    ADD CONSTRAINT tagmessage_user_fk FOREIGN KEY (created_by) REFERENCES public."user"(id);


-- Completed on 2025-01-02 22:42:04 CET

--
-- PostgreSQL database dump complete
--

